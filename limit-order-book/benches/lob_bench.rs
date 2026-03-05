use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use limit_order_book::{LimitOrderBook, Side};

const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;
const DEPTHS: &[u64] = &[10, 100, 1_000];

fn prefilled_book(num_levels: u64) -> LimitOrderBook {
    let mut book = LimitOrderBook::new();
    let mut id = 1u64;

    for lvl in 0..num_levels {
        for _ in 0..ORDERS_PER_LEVEL {
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
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d),
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
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d),
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
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d),
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
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d),
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
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d),
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
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || (prefilled_book(d), 1u64),
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
    for &depth in DEPTHS {
        let book = prefilled_book(depth);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, _| {
            b.iter(|| black_box(book.spread()));
        });
    }
    group.finish();
}

fn bench_add_cancel_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_then_cancel_cycle");
    for &depth in DEPTHS {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d),
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
);
criterion_main!(benches);
