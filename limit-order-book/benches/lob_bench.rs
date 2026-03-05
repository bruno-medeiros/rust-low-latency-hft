use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use limit_order_book::{LimitOrderBook, Side};

fn prefilled_book(num_levels: u64, orders_per_level: u64) -> LimitOrderBook {
    let mut book = LimitOrderBook::new();
    let mut id = 1u64;
    let mid = 10_000u64;

    for lvl in 0..num_levels {
        for _ in 0..orders_per_level {
            book.add_limit_order(id, Side::Buy, mid - 1 - lvl, 100);
            id += 1;
            book.add_limit_order(id, Side::Sell, mid + 1 + lvl, 100);
            id += 1;
        }
    }
    book
}

fn bench_add_limit_order_passive(c: &mut Criterion) {
    c.bench_function("add_limit_order/passive", |b| {
        b.iter_batched(
            || prefilled_book(100, 10),
            |mut book| {
                black_box(book.add_limit_order(999_999, Side::Buy, 5_000, 50));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_add_limit_order_match_single(c: &mut Criterion) {
    c.bench_function("add_limit_order/match_single_fill", |b| {
        b.iter_batched(
            || prefilled_book(100, 10),
            |mut book| {
                black_box(book.add_limit_order(999_999, Side::Buy, 10_001, 50));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_add_limit_order_sweep_5_levels(c: &mut Criterion) {
    c.bench_function("add_limit_order/sweep_5_levels", |b| {
        b.iter_batched(
            || prefilled_book(100, 10),
            |mut book| {
                black_box(book.add_limit_order(999_999, Side::Buy, 10_005, 5_000));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_add_market_order(c: &mut Criterion) {
    c.bench_function("add_market_order/full_fill", |b| {
        b.iter_batched(
            || prefilled_book(100, 10),
            |mut book| {
                black_box(book.add_market_order(999_999, Side::Buy, 50));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_add_market_order_sweep(c: &mut Criterion) {
    c.bench_function("add_market_order/sweep_10_levels", |b| {
        b.iter_batched(
            || prefilled_book(100, 10),
            |mut book| {
                black_box(book.add_market_order(999_999, Side::Buy, 10_000));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_cancel_order(c: &mut Criterion) {
    c.bench_function("cancel_order", |b| {
        b.iter_batched(
            || {
                let book = prefilled_book(100, 10);
                let target_id = 1u64;
                (book, target_id)
            },
            |(mut book, id)| {
                black_box(book.cancel_order(id));
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_spread(c: &mut Criterion) {
    let book = prefilled_book(100, 10);
    c.bench_function("spread", |b| {
        b.iter(|| black_box(book.spread()));
    });
}

fn bench_add_cancel_cycle(c: &mut Criterion) {
    c.bench_function("add_then_cancel_cycle", |b| {
        b.iter_batched(
            || prefilled_book(100, 10),
            |mut book| {
                book.add_limit_order(999_999, Side::Buy, 9_500, 100);
                black_box(book.cancel_order(999_999));
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_add_limit_order_passive,
    bench_add_limit_order_match_single,
    bench_add_limit_order_sweep_5_levels,
    bench_add_market_order,
    bench_add_market_order_sweep,
    bench_cancel_order,
    bench_spread,
    bench_add_cancel_cycle,
);
criterion_main!(benches);
