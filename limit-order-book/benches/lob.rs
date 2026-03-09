use std::alloc::System;
use std::hint::black_box;

use bench_tool::{BenchRunner, CliArgs, StatsAlloc, INSTRUMENTED_SYSTEM};
use limit_order_book::{LimitOrderBook, Side};
use limit_order_book::EventKind::Fill;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const NUM_LEVELS: u64 = 100;
const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;

// A single sell-side price level with many resting orders.
// Used to measure cancel cost as a function of queue position.
const CROWDED_LEVEL_ORDERS: u64 = 500;

fn prefilled_book() -> LimitOrderBook {
    let mut book = LimitOrderBook::new();
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
fn crowded_sell_level() -> LimitOrderBook {
    let mut book = LimitOrderBook::new();
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
        .param("crowded_level_orders", &CROWDED_LEVEL_ORDERS.to_string());

    // ── Commands ──────────────────────────────────────────────────────────────

    runner.run("Add (passive)", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, 5_000, 50);
    });

    runner.run("Add (single fill)", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, MID_PRICE + 1, 50);
    });

    // Walks 5 sell levels (MID_PRICE+1 … +5);
    runner.run("Add (sweep 5 levels, 50 fills)", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, MID_PRICE + 5, 5_000);
    });

    // Consumes qty 10 000 across 10 sell levels; 10 × 10 orders = 100 fills.
    {
        let mut book = prefilled_book();
        let vec = book.add_market_order(999_999, Side::Buy, 10_000);
        assert_eq!(vec.iter().filter(|e| matches!(e.kind, Fill {..})).count(), 100);
    }
    runner.run(
        "Market (sweep 10 levels, 100 fills)",
        GLOBAL,
        prefilled_book,
        |book| {
            book.add_market_order(999_999, Side::Buy, 10_000);
        },
    );

    // Order 1 is the first enqueued at its price level — O(1) VecDeque pop.
    runner.run("Cancel (head of queue)", GLOBAL, prefilled_book, |book| {
        book.cancel_order(1);
    });

    // Order CROWDED_LEVEL_ORDERS is the last enqueued in a 500-order level.
    // PriceLevel::remove is an O(n) linear scan; this shows worst-case cost.
    runner.run(
        "Cancel (tail of queue)",
        GLOBAL,
        crowded_sell_level,
        |book| {
            book.cancel_order(CROWDED_LEVEL_ORDERS);
        },
    );   

    // ── Queries ───────────────────────────────────────────────────────────────

    // Calls best_bid + best_ask — BTreeMap::last/first_key_value, O(log n) but
    // practically O(1) with a hot cache.
    runner.run("Spread (BBO query)", GLOBAL, prefilled_book, |book| {
        black_box(book.spread());
    });

    // Returns top 5 levels as a Vec — allocates on every call.
    runner.run("Depth (top 5)", GLOBAL, prefilled_book, |book| {
        black_box(book.depth(Side::Sell, 5));
    });

    // HashMap::get — O(1) average.
    runner.run("Order lookup (hit)", GLOBAL, prefilled_book, |book| {
        black_box(book.order(1));
    });  

    // ── Realistic mix ────────────────────────────────────────────────────────
    // Cycles through a 10-op sequence that approximates a live trading workload:
    //   40% passive limit adds
    //   30% cancels
    //   20% aggressive matches
    //   10% BBO queries
    //
    // Each sample runs one operation from the cycle on a fresh prefilled book.
    // The histogram therefore captures the latency *distribution* across the
    // mix, not just a single operation type.
    let mut mix_cursor = 0usize;
    runner.run("Realistic mix (per-op)", GLOBAL, prefilled_book, |book| {
        let idx = mix_cursor % 10;
        mix_cursor += 1;
        match idx {
            0..=3 => {
                book.add_limit_order(999_990 + idx as u64, Side::Buy, 5_000 + idx as u64, 50);
            }
            4..=6 => {
                // Cancel buy orders 1, 3, 5 — each at the head of their level.
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
    });

    let report = runner.finish();
    args.execute(&report)
}
