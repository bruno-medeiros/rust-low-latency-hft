use std::hint::black_box;

use bench_tool::{BenchRunner, CliArgs, RunMode};
use limit_order_book::LimitOrderBookV0;
use limit_order_book::event::EventKind::Fill;
use limit_order_book::types::Side;

const NUM_LEVELS: u64 = 100;
const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;

/// Single price level with many resting orders; measures cancel cost vs queue position.
const CROWDED_LEVEL_ORDERS: u64 = 500;

const BENCH_ITERS: u64 = 100_000;

fn prefilled_book() -> LimitOrderBookV0 {
    let mut book = LimitOrderBookV0::new();
    let mut id = 1u64;
    for lvl in 0..NUM_LEVELS {
        for _ in 0..ORDERS_PER_LEVEL {
            book.add_limit_order(id, Side::Buy, MID_PRICE - 1 - lvl, 100);
            id += 1;
            book.add_limit_order(id, Side::Sell, MID_PRICE + 1 + lvl, 100);
            id += 1;
        }
    }
    book
}

// One sell-side price level with CROWDED_LEVEL_ORDERS resting orders.
// Order IDs 1..=CROWDED_LEVEL_ORDERS, enqueued in arrival order.
fn crowded_sell_level() -> LimitOrderBookV0 {
    let mut book = LimitOrderBookV0::new();
    for id in 1..=CROWDED_LEVEL_ORDERS {
        book.add_limit_order(id, Side::Sell, MID_PRICE + 1, 100);
    }
    book
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse_args();

    let mut runner = BenchRunner::new("Limit Order Book \u{2014} Latency")
        .warmup_iters(10_000)
        .sample_iters(100_000)
        .param("book_levels", &NUM_LEVELS.to_string())
        .param("orders_per_level", &ORDERS_PER_LEVEL.to_string())
        .param(
            "resting_orders",
            &(NUM_LEVELS * ORDERS_PER_LEVEL * 2).to_string(),
        )
        .param("crowded_level_orders", &CROWDED_LEVEL_ORDERS.to_string())
        .param("iters", &BENCH_ITERS.to_string());

    // ── Commands ──────────────────────────────────────────────────────────────

    // Limit order that rests — no match.
    runner.run(
        RunMode::Latency,
        "Add (passive)",
        prefilled_book,
        |book| {
            book.add_limit_order(999_999, Side::Buy, 5_000, 50);
        },
        BENCH_ITERS,
    );

    // Aggressive order crossing 5 sell levels; fills 50 resting orders.
    runner.run(
        RunMode::Latency,
        "Add (sweep 5 levels, 50 fills)",
        prefilled_book,
        |book| {
            book.add_limit_order(999_999, Side::Buy, MID_PRICE + 5, 5_000);
        },
        BENCH_ITERS,
    );

    // Market order consuming 10 levels × 10 orders = 100 fills.
    {
        let mut book = prefilled_book();
        let vec = book.add_market_order(999_999, Side::Buy, 10_000);
        assert_eq!(
            vec.iter().filter(|e| matches!(e.kind, Fill { .. })).count(),
            100
        );
    }
    runner.run(
        RunMode::Latency,
        "Market (sweep 10 levels, 100 fills)",
        prefilled_book,
        |book| {
            book.add_market_order(999_999, Side::Buy, 10_000);
        },
        BENCH_ITERS,
    );

    // Cancel the first order at a price level — best-case queue position.
    runner.run(
        RunMode::Latency,
        "Cancel (head of queue)",
        prefilled_book,
        |book| {
            book.cancel_order(1);
        },
        BENCH_ITERS,
    );

    // Cancel the last order in a deep queue — worst-case queue position.
    runner.run(
        RunMode::Latency,
        "Cancel (tail of queue)",
        crowded_sell_level,
        |book| {
            book.cancel_order(CROWDED_LEVEL_ORDERS);
        },
        BENCH_ITERS,
    );

    // ── Queries ───────────────────────────────────────────────────────────────

    // Best bid + best ask — primary read path for both sides.
    runner.run(
        RunMode::Latency,
        "Spread (BBO query)",
        prefilled_book,
        |book| {
            black_box(book.spread());
        },
        BENCH_ITERS,
    );

    // Top-N levels query; returns aggregate quantity per level.
    runner.run(
        RunMode::Latency,
        "Depth (top 5)",
        prefilled_book,
        |book| {
            black_box(book.depth(Side::Sell, 5));
        },
        BENCH_ITERS,
    );

    // Lookup resting order by ID.
    runner.run(
        RunMode::Latency,
        "Order lookup (hit)",
        prefilled_book,
        |book| {
            black_box(book.order(1));
        },
        BENCH_ITERS,
    );

    // ── Realistic mix (per-op latency) ────────────────────────────────────────
    // 40% passive add / 30% cancel / 20% match / 10% BBO. Each sample runs one
    // op from the cycle on a fresh book; histogram captures latency distribution.
    let mut mix_cursor = 0usize;
    runner.run(
        RunMode::Latency,
        "Realistic mix (per-op)",
        prefilled_book,
        |book| {
            let idx = mix_cursor % 10;
            mix_cursor += 1;
            match idx {
                0..=3 => {
                    book.add_limit_order(999_990 + idx as u64, Side::Buy, 5_000 + idx as u64, 50);
                }
                4..=6 => {
                    book.cancel_order(1 + (idx as u64 - 4) * 2);
                }
                7..=8 => {
                    book.add_limit_order(
                        999_990 + idx as u64,
                        Side::Buy,
                        MID_PRICE + 1 + (idx as u64 - 7),
                        50,
                    );
                }
                _ => {
                    black_box(book.best_bid());
                }
            }
        },
        BENCH_ITERS,
    );

    // ── Throughput (sustained mix) ────────────────────────────────────────────
    // Same mix as above, but runs on a single book in a tight loop. Sustainable:
    // 4 add + 4 cancel (paired) + 2 BBO per 10 ops.
    struct ThroughputState {
        book: LimitOrderBookV0,
        cycle: usize,
    }
    runner.run(
        RunMode::Throughput,
        "Throughput (sustained mix)",
        || ThroughputState {
            book: prefilled_book(),
            cycle: 0,
        },
        |state| {
            let base = 100_000 + state.cycle * 10;
            for i in 0..4 {
                state
                    .book
                    .add_limit_order((base + i) as u64, Side::Buy, 5_000 + i as u64, 50);
            }
            for i in 0..4 {
                state.book.cancel_order((base + i) as u64);
            }
            black_box(state.book.spread());
            black_box(state.book.spread());
            state.cycle += 1;
        },
        BENCH_ITERS,
    );

    let report = runner.finish();
    args.execute(&report)
}
