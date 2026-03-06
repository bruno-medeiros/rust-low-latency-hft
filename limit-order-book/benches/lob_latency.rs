use std::alloc::System;

use bench_tool::{BenchRunner, CliArgs, StatsAlloc, INSTRUMENTED_SYSTEM};
use limit_order_book::{LimitOrderBook, Side};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const NUM_LEVELS: u64 = 100;
const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;

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
        );

    runner.run("Add (passive)", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, 5_000, 50);
    });

    runner.run("Add (match, 1 fill)", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, MID_PRICE + 1, 50);
    });

    runner.run("Add (sweep 5 levels)", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, MID_PRICE + 5, 5_000);
    });

    runner.run(
        "Market order (full fill)",
        GLOBAL,
        prefilled_book,
        |book| {
            book.add_market_order(999_999, Side::Buy, 50);
        },
    );

    runner.run(
        "Market order (sweep 10)",
        GLOBAL,
        prefilled_book,
        |book| {
            book.add_market_order(999_999, Side::Buy, 10_000);
        },
    );

    runner.run("Cancel", GLOBAL, prefilled_book, |book| {
        book.cancel_order(1);
    });

    runner.run("Spread (BBO query)", GLOBAL, prefilled_book, |book| {
        book.spread();
    });

    runner.run("Add + Cancel cycle", GLOBAL, prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, 9_500, 100);
        book.cancel_order(999_999);
    });

    let report = runner.finish();
    args.execute(&report)
}
