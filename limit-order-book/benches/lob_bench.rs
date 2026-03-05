use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use limit_order_book::{LimitOrderBook, Side};

const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;

fn prefilled_book(num_levels: u64, orders_per_level: u64) -> LimitOrderBook {
    let mut book = LimitOrderBook::new();
    let mut id = 1u64;

    for lvl in 0..num_levels {
        for _ in 0..orders_per_level {
            book.add_limit_order(id, Side::Buy, MID_PRICE - 1 - lvl, 100);
            id += 1;
            book.add_limit_order(id, Side::Sell, MID_PRICE + 1 + lvl, 100);
            id += 1;
        }
    }
    book
}

fn bench_add_passive(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_limit_order/passive");
    for (depth, orders_per_level, label) in [
        (10, 10, "10x10"),
        (100, 100, "100x100"),
        (10, 1000, "10x1000"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, orders_per_level),
                |mut book| {
                    black_box(book.add_limit_order(999_999, Side::Buy, 5_000, 50));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_add_match_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_limit_order/match_single_fill");
    for (depth, orders_per_level, label) in [
        (10, 10, "10x10"),
        (100, 100, "100x100"),
        (10, 1000, "10x1000"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, orders_per_level),
                |mut book| {
                    black_box(book.add_limit_order(999_999, Side::Buy, MID_PRICE + 1, 50));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_add_sweep_5(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_limit_order/sweep_5_levels");
    for (depth, label) in [(10, "10"), (100, "100"), (1000, "1000")] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, ORDERS_PER_LEVEL),
                |mut book| {
                    black_box(book.add_limit_order(999_999, Side::Buy, MID_PRICE + 5, 5_000));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_market_full_fill(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_market_order/full_fill");
    for (depth, orders_per_level, label) in [(10, 10, "10x10")] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, orders_per_level),
                |mut book| {
                    black_box(book.add_market_order(999_999, Side::Buy, 50));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_market_sweep_10(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_market_order/sweep_10_levels");
    for (depth, orders_per_level, label) in [(10, 10, "10x10"), (10, 1000, "10x1000")] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, orders_per_level),
                |mut book| {
                    black_box(book.add_market_order(999_999, Side::Buy, 10_000));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_cancel(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancel_order");
    for (depth, orders_per_level, label) in [(10, 10, "10x10"), (100, 1000, "100x1000")] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || (prefilled_book(d, orders_per_level), 1u64),
                |(mut book, id)| {
                    black_box(book.cancel_order(id));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_spread(c: &mut Criterion) {
    let mut group = c.benchmark_group("spread");
    for (depth, label) in [(10, "10x10"), (100, "100x10"), (1000, "1000x10")] {
        let book = prefilled_book(depth, ORDERS_PER_LEVEL);
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, _| {
            b.iter(|| black_box(book.spread()));
        });
    }
    group.finish();
}

fn bench_add_cancel_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_then_cancel_cycle");
    for (depth, orders_per_level, label) in [
        (10, 10, "10x10"),
        (100, 100, "100x100"),
        (10, 1000, "10x1000"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, orders_per_level),
                |mut book| {
                    book.add_limit_order(999_999, Side::Buy, 9_500, 100);
                    black_box(book.cancel_order(999_999));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// --- Worst-case scenarios ---

fn bench_cancel_deepest(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst/cancel_deepest");
    for (depth, label) in [(10, "10x10"), (100, "100x10")] {
        let last_buy_id = depth * ORDERS_PER_LEVEL * 2 - 1;
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, ORDERS_PER_LEVEL),
                |mut book| {
                    black_box(book.cancel_order(last_buy_id));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_sweep_full_book(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst/sweep_full_book");
    for (depth, label) in [(10, "10x10"), (100, "100x10")] {
        let total_ask_qty = depth * ORDERS_PER_LEVEL * 100;
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, ORDERS_PER_LEVEL),
                |mut book| {
                    black_box(book.add_market_order(999_999, Side::Buy, total_ask_qty));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_add_to_crowded_level(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst/add_to_crowded_level");
    for (crowd, label) in [(100, "100ord"), (1000, "1Kord"), (10_000, "10Kord")] {
        group.bench_with_input(BenchmarkId::new("queue", label), &crowd, |b, &n| {
            b.iter_batched(
                || prefilled_book(1, n),
                |mut book| {
                    black_box(book.add_limit_order(999_999, Side::Buy, MID_PRICE - 1, 50));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_add_passive,
    bench_add_match_single,
    bench_add_sweep_5,
    bench_market_full_fill,
    bench_market_sweep_10,
    bench_cancel,
    bench_spread,
    bench_add_cancel_cycle,
    bench_cancel_deepest,
    bench_sweep_full_book,
    bench_add_to_crowded_level,
);
criterion_main!(benches);
