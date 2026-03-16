use std::hint::black_box;

use bench_tool::{BenchRunner, CliArgs};
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

    runner.run_latency(
        "Add (passive)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.add_limit_order(OP_ORDER_ID, Side::Buy, 5_000, 50, &mut sink);
        },
        BENCH_ITERS,
    );

    runner.run_latency(
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
    runner.run_latency(
        "Market (sweep 10 levels, 100 fills)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.add_market_order(OP_ORDER_ID, Side::Buy, 10_000, &mut sink);
        },
        BENCH_ITERS,
    );

    // Cancel the first order at a price level — best-case queue position.
    runner.run_latency(
        "Cancel (head of queue)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            let mut sink = CountingEventSink::default();
            book.cancel_order(1, &mut sink);
        },
        BENCH_ITERS,
    );

    // Cancel the last order in a deep queue — worst-case queue position.
    runner.run_latency(
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
    runner.run_latency(
        "Spread (BBO query)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            black_box(book.spread());
        },
        BENCH_ITERS,
    );

    // Top-N levels query; returns aggregate quantity per level.
    runner.run_latency(
        "Depth (top 5)",
        || prefilled_book(PRICE_RANGE, ORDER_CAPACITY),
        |book| {
            black_box(book.depth(Side::Sell, 5));
        },
        BENCH_ITERS,
    );

    // Lookup resting order by ID.
    runner.run_latency(
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
    runner.run_latency(
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
    // Realistic mix: passive adds (unfilled), aggressive adds (multiple fills), cancels, spread.
    // Per cycle: 20 passive buy, 20 passive sell, 8 aggressive buy (hit our sells), 20 cancel buy,
    // 8 cancel sell (remaining after match), 12 spread. Steady state: aggressors hit our passive
    // sells; new order IDs every cycle via op_id_counter.
    const CANCEL_PER_CYCLE: u64 = 30;
    const PASSIVE_PER_CYCLE: u64 = 20;
    const AGGRESSIVE_PER_CYCLE: u64 = 8;
    const SPREAD_PER_CYCLE: u64 = 12;
    const RESTING_QTY: u64 = 100;
    const AGGRESSIVE_QTY: u64 = 150; // > RESTING_QTY so each aggressor gets 2+ Fill events

    struct ThroughputState<T: LimitOrderBook> {
        book: T,
        order_id_counter: u64,
    }
    let _state = runner.run_throughput(
        "Throughput (sustained mix)",
        || ThroughputState {
            book: prefilled_book(PRICE_RANGE, 150_000_000),
            order_id_counter: OP_ORDER_ID_BASE,
        },
        |state, sink, op_count| {
            let initial_order_count = state.book.order_count();
            let base_orders_to_cancel = state.order_id_counter;
            // Passive buy (rest at 5000..5019)
            for i in 0..CANCEL_PER_CYCLE {
                state.book.add_limit_order(
                    state.order_id_counter + i,
                    Side::Buy,
                    5_000 + i,
                    RESTING_QTY,
                    sink,
                );
            }
            state.order_id_counter += CANCEL_PER_CYCLE;
            *op_count += CANCEL_PER_CYCLE;

            // Passive sell at best ask so aggressors can hit them (steady state, no prefilled drain)
            let base_passive = state.order_id_counter;
            for i in 0..PASSIVE_PER_CYCLE {
                state.book.add_limit_order(
                    state.order_id_counter + i,
                    Side::Sell,
                    MID_PRICE,
                    RESTING_QTY,
                    sink,
                );
            }
            state.order_id_counter += PASSIVE_PER_CYCLE;
            *op_count += PASSIVE_PER_CYCLE;

            // Aggressive buy (multiple fills per order; hit our passive sells)
            for i in 0..AGGRESSIVE_PER_CYCLE {
                state.book.add_limit_order(
                    state.order_id_counter + i,
                    Side::Buy,
                    MID_PRICE,
                    AGGRESSIVE_QTY,
                    sink,
                );
            }
            state.order_id_counter += AGGRESSIVE_PER_CYCLE;
            *op_count += AGGRESSIVE_PER_CYCLE;

            // Run some spreads
            for _ in 0..SPREAD_PER_CYCLE {
                black_box(state.book.spread());
            }
            *op_count += SPREAD_PER_CYCLE;

            // Cancel all passive buy we added this cycle
            for i in 0..CANCEL_PER_CYCLE {
                state.book.cancel_order(base_orders_to_cancel + i, sink);
            }
            *op_count += CANCEL_PER_CYCLE;

            // Cancel the 8 passive sells that still have remaining qty (first 12 were fully filled by aggressors)
            for i in 12..PASSIVE_PER_CYCLE {
                state.book.cancel_order(base_passive + i, sink);
            }
            *op_count += PASSIVE_PER_CYCLE;

            // Check steady order count.
            assert!(state.book.order_count() == initial_order_count);
        },
        2_000_000,
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

    // TODO: add to params?
    runner.apply_core_pinning();

    match version.as_str() {
        "v0" => run_benchmarks::<LimitOrderBookV0>(&mut runner),
        "v1" => run_benchmarks::<LimitOrderBookV1>(&mut runner),
        _ => return Err(format!("unknown LOB version: {version}; expected v0 or v1").into()),
    }

    let report = runner.finish();
    args.execute(&report)
}
