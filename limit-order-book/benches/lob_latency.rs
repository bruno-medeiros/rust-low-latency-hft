use std::hint::black_box;
use std::time::Instant;

use hdrhistogram::Histogram;
use limit_order_book::{LimitOrderBook, Side};

const WARMUP_ITERS: u64 = 10_000;
const SAMPLE_ITERS: u64 = 100_000;
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

struct Scenario {
    name: &'static str,
    run: fn() -> Histogram<u64>,
}

fn record<S, F>(setup: S, mut op: F) -> Histogram<u64>
where
    S: Fn() -> LimitOrderBook,
    F: FnMut(&mut LimitOrderBook),
{
    // Highest trackable value: 1 second in nanos. 3 significant digits.
    let mut hist = Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).unwrap();

    for _ in 0..WARMUP_ITERS {
        let mut book = setup();
        black_box(op(&mut book));
    }

    for _ in 0..SAMPLE_ITERS {
        let mut book = setup();

        let start = Instant::now();
        black_box(op(&mut book));
        let elapsed_ns = start.elapsed().as_nanos() as u64;

        hist.record(elapsed_ns).unwrap();
    }

    hist
}

fn print_report(name: &str, hist: &Histogram<u64>) {
    println!(
        "  {:<35} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        name,
        fmt_duration(hist.min()),
        fmt_duration(hist.value_at_quantile(0.50)),
        fmt_duration(hist.value_at_quantile(0.95)),
        fmt_duration(hist.value_at_quantile(0.99)),
        fmt_duration(hist.value_at_quantile(0.999)),
        fmt_duration(hist.max()),
    );
}

fn fmt_duration(ns: u64) -> String {
    if ns >= 1_000_000 {
        format!("{:.1}ms", ns as f64 / 1_000_000.0)
    } else if ns >= 1_000 {
        format!("{:.1}μs", ns as f64 / 1_000.0)
    } else {
        format!("{}ns", ns)
    }
}

fn scenario_add_passive() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, 5_000, 50);
    })
}

fn scenario_add_match_single() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, MID_PRICE + 1, 50);
    })
}

fn scenario_add_sweep_5() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, MID_PRICE + 5, 5_000);
    })
}

fn scenario_market_full_fill() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.add_market_order(999_999, Side::Buy, 50);
    })
}

fn scenario_market_sweep_10() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.add_market_order(999_999, Side::Buy, 10_000);
    })
}

fn scenario_cancel() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.cancel_order(1);
    })
}

fn scenario_spread() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.spread();
    })
}

fn scenario_add_cancel_cycle() -> Histogram<u64> {
    record(prefilled_book, |book| {
        book.add_limit_order(999_999, Side::Buy, 9_500, 100);
        book.cancel_order(999_999);
    })
}

fn main() {
    let scenarios: Vec<Scenario> = vec![
        Scenario {
            name: "Add (passive)",
            run: scenario_add_passive,
        },
        Scenario {
            name: "Add (match, 1 fill)",
            run: scenario_add_match_single,
        },
        Scenario {
            name: "Add (sweep 5 levels)",
            run: scenario_add_sweep_5,
        },
        Scenario {
            name: "Market order (full fill)",
            run: scenario_market_full_fill,
        },
        Scenario {
            name: "Market order (sweep 10)",
            run: scenario_market_sweep_10,
        },
        Scenario {
            name: "Cancel",
            run: scenario_cancel,
        },
        Scenario {
            name: "Spread (BBO query)",
            run: scenario_spread,
        },
        Scenario {
            name: "Add + Cancel cycle",
            run: scenario_add_cancel_cycle,
        },
    ];

    println!();
    println!(
        "  Limit Order Book — Latency Distribution ({} samples per scenario)",
        SAMPLE_ITERS
    );
    println!(
        "  Book: {} levels × {} orders/level ({} resting orders)",
        NUM_LEVELS,
        ORDERS_PER_LEVEL,
        NUM_LEVELS * ORDERS_PER_LEVEL * 2
    );
    println!();
    println!(
        "  {:<35} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Operation", "min", "p50", "p95", "p99", "p99.9", "max"
    );
    println!("  {}", "─".repeat(99));

    for s in &scenarios {
        let hist = (s.run)();
        print_report(s.name, &hist);
    }

    println!();
}
