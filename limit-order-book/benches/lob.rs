use std::hint::black_box;

use bench_tool::{BenchRunner, CliArgs, RunMode};
use limit_order_book::types::Side;
use limit_order_book::{CountingEventSink, LimitOrderBook, LimitOrderBookV0, LimitOrderBookV1};

const NUM_LEVELS: u64 = 100;
const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;

/// Single price level with many resting orders; measures cancel cost vs queue position.
const CROWDED_LEVEL_ORDERS: u64 = 500;

const BENCH_ITERS: u64 = 100_000;

const PRICE_RANGE: (u64, u64) = (1, 20_000);
const ORDER_CAPACITY: u64 = 10_000;
const THROUGHPUT_ORDER_CAPACITY: u64 = 500_000;

/// Order IDs used by benchmark operations (above the prefilled range 1..=2000).
const OP_ORDER_ID: u64 = 3_000;
const OP_ORDER_ID_BASE: u64 = 3_000;

fn fill_book(book: &mut impl LimitOrderBook) {
    let mut sink = CountingEventSink::default();
    let mut id = 1u64;
    for lvl in 0..NUM_LEVELS {
        for _ in 0..ORDERS_PER_LEVEL {
            book.add_limit_order(id, Side::Buy, MID_PRICE - 1 - lvl, 100, &mut sink);
            id += 1;
            book.add_limit_order(id, Side::Sell, MID_PRICE + 1 + lvl, 100, &mut sink);
            id += 1;
        }
    }
}

fn fill_crowded_sell(book: &mut impl LimitOrderBook) {
    let mut sink = CountingEventSink::default();
    for id in 1..=CROWDED_LEVEL_ORDERS {
        book.add_limit_order(id, Side::Sell, MID_PRICE + 1, 100, &mut sink);
    }
}

fn run_benchmarks<B: LimitOrderBook>(runner: &mut BenchRunner) {
    let prefilled_book = |price_range, order_capacity| {
        let mut b = B::with_config(price_range, order_capacity);
        fill_book(&mut b);
        b
    };
    let crowded_sell_level = |price_range, order_capacity| {
        let mut b = B::with_config(price_range, order_capacity);
        fill_crowded_sell(&mut b);
        b
    };

    // ── Commands ──────────────────────────────────────────────────────────────

    runner.run(
        RunMode::Latency,
        "Add (passive)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.add_limit_order(OP_ORDER_ID, Side::Buy, 5_000, 50, &mut sink);
        },
        BENCH_ITERS,
    );

    runner.run(
        RunMode::Latency,
        "Add (sweep 5 levels, 50 fills)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.add_limit_order(OP_ORDER_ID, Side::Buy, MID_PRICE + 5, 5_000, &mut sink);
        },
        BENCH_ITERS,
    );

    // Verify the market-order scenario produces the expected number of fills.
    {
        let mut book = prefilled_book(PRICE_RANGE, ORDER_CAPACITY);
        let mut sink = CountingEventSink::default();
        book.add_market_order(OP_ORDER_ID, Side::Buy, 10_000, &mut sink);
        assert_eq!(sink.fill, 100);
    }
    runner.run(
        RunMode::Latency,
        "Market (sweep 10 levels, 100 fills)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.add_market_order(OP_ORDER_ID, Side::Buy, 10_000, &mut sink);
        },
        BENCH_ITERS,
    );

    // Cancel the first order at a price level — best-case queue position.
    runner.run(
        RunMode::Latency,
        "Cancel (head of queue)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.cancel_order(1, &mut sink);
        },
        BENCH_ITERS,
    );

    // Cancel the last order in a deep queue — worst-case queue position.
    runner.run(
        RunMode::Latency,
        "Cancel (tail of queue)",
        || crowded_sell_level(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.cancel_order(CROWDED_LEVEL_ORDERS, &mut sink);
        },
        BENCH_ITERS,
    );

    // ── Queries ───────────────────────────────────────────────────────────────

    // Best bid + best ask — primary read path for both sides.
    runner.run(
        RunMode::Latency,
        "Spread (BBO query)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            black_box(book.spread());
        },
        BENCH_ITERS,
    );

    // Top-N levels query; returns aggregate quantity per level.
    runner.run(
        RunMode::Latency,
        "Depth (top 5)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            black_box(book.depth(Side::Sell, 5));
        },
        BENCH_ITERS,
    );

    // Lookup resting order by ID.
    runner.run(
        RunMode::Latency,
        "Order lookup (hit)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            black_box(book.order(1));
        },
        BENCH_ITERS,
    );

    // ── Realistic mix (per-op latency) ────────────────────────────────────────
    // 40% passive add / 30% cancel / 20% match / 10% BBO.
    let mut mix_cursor = 0usize;
    runner.run(
        RunMode::Latency,
        "Realistic mix (per-op)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            let idx = mix_cursor % 10;
            mix_cursor += 1;
            match idx {
                0..=3 => {
                    book.add_limit_order(
                        OP_ORDER_ID_BASE + idx as u64,
                        Side::Buy,
                        5_000 + idx as u64,
                        50,
                        &mut sink,
                    );
                }
                4..=6 => {
                    book.cancel_order(1 + (idx as u64 - 4) * 2, &mut sink);
                }
                7..=8 => {
                    book.add_limit_order(
                        OP_ORDER_ID_BASE + idx as u64,
                        Side::Buy,
                        MID_PRICE + 1 + (idx as u64 - 7),
                        50,
                        &mut sink,
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
    // 4 add + 4 cancel (paired) + 2 BBO per 10 ops.
    // Uses a larger order_capacity to stress the V1 data structure more realistically.
    // IDs are reused each cycle since all adds are cancelled within the same iteration.
    struct ThroughputState<T: LimitOrderBook> {
        book: T,
        sink: CountingEventSink,
        cycle: usize,
    }
    runner.run(
        RunMode::Throughput,
        "Throughput (sustained mix)",
        || ThroughputState {
            book: prefilled_book(PRICE_RANGE, THROUGHPUT_ORDER_CAPACITY),
            sink: CountingEventSink::default(),
            cycle: 0,
        },
        |state| {
            let base = OP_ORDER_ID_BASE as usize + state.cycle * 10;
            for i in 0..4 {
                state.book.add_limit_order(
                    (base + i) as u64,
                    Side::Buy,
                    5_000 + i as u64,
                    50,
                    &mut state.sink,
                );
            }
            for i in 0..4 {
                state.book.cancel_order((base + i) as u64, &mut state.sink);
            }
            black_box(state.book.spread());
            black_box(state.book.spread());
            state.cycle += 1;
        },
        BENCH_ITERS,
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse_args();
    let version = &args.lob_version;

    let mut runner = BenchRunner::new(&format!("Limit Order Book ({version}) \u{2014} Latency"))
        .warmup_iters(10_000)
        .sample_iters(100_000)
        .filter(args.filter.clone())
        .param("book_levels", &NUM_LEVELS.to_string())
        .param("orders_per_level", &ORDERS_PER_LEVEL.to_string())
        .param(
            "resting_orders",
            &(NUM_LEVELS * ORDERS_PER_LEVEL * 2).to_string(),
        )
        .param("crowded_level_orders", &CROWDED_LEVEL_ORDERS.to_string())
        .param("iters", &BENCH_ITERS.to_string())
        .param("lob_version", version);

    match version.as_str() {
        "v0" => run_benchmarks::<LimitOrderBookV0>(&mut runner),
        "v1" => run_benchmarks::<LimitOrderBookV1>(&mut runner),
        _ => return Err(format!("unknown LOB version: {version}; expected v0 or v1").into()),
    }

    let report = runner.finish();
    args.execute(&report)
}
