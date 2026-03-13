use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use limit_order_book::types::Side;
use limit_order_book::{CountingEventSink, LimitOrderBookV0};

pub const ORDERS_PER_LEVEL: u64 = 10;
const MID_PRICE: u64 = 10_000;

fn prefilled_book(num_levels: u64, orders_per_level: u64) -> LimitOrderBookV0 {
    let mut book = LimitOrderBookV0::new();
    let mut sink = CountingEventSink::default();
    let mut id = 1u64;

    for lvl in 0..num_levels {
        for _ in 0..orders_per_level {
            book.add_limit_order(id, Side::Buy, MID_PRICE - 1 - lvl, 100, &mut sink);
            id += 1;
            book.add_limit_order(id, Side::Sell, MID_PRICE + 1 + lvl, 100, &mut sink);
            id += 1;
        }
    }
    book
}

// Walks MID_PRICE+1 … +5 (5 levels), filling ORDERS_PER_LEVEL orders per level.
// Total fills per call: 5 × ORDERS_PER_LEVEL = 50, regardless of book depth.
// Depth variants show how BTreeMap traversal time scales with the number of levels.
fn bench_add_sweep_5(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_limit_order/sweep_5_levels");
    for (depth, label) in [
        (10, "10/50fills"),
        (100, "100/50fills"),
        (1000, "1000/50fills"),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &depth, |b, &d| {
            b.iter_batched(
                || prefilled_book(d, ORDERS_PER_LEVEL),
                |mut book| {
                    let mut sink = CountingEventSink::default();
                    #[allow(clippy::unit_arg)]
                    black_box(book.add_limit_order(
                        999_999,
                        Side::Buy,
                        MID_PRICE + 5,
                        5_000,
                        &mut sink,
                    ));
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// Check scaling issues with PriceLevel::remove
fn bench_cancel_queue_position(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancel_order/");

    for (queue_len, label) in [(100u64, "100"), (500u64, "500"), (1_000u64, "1000")] {
        let setup = move || {
            let mut book = LimitOrderBookV0::new();
            let mut sink = CountingEventSink::default();
            for id in 1..=queue_len {
                book.add_limit_order(id, Side::Sell, MID_PRICE + 1, 100, &mut sink);
            }
            book
        };

        group.bench_with_input(BenchmarkId::new("head", label), &queue_len, |b, _| {
            b.iter_batched(
                setup,
                |mut book| {
                    let mut sink = CountingEventSink::default();
                    #[allow(clippy::unit_arg)]
                    black_box(book.cancel_order(1, &mut sink));
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("tail", label), &queue_len, |b, &n| {
            b.iter_batched(
                setup,
                |mut book| {
                    let mut sink = CountingEventSink::default();
                    #[allow(clippy::unit_arg)]
                    black_box(book.cancel_order(n, &mut sink));
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_add_sweep_5, bench_cancel_queue_position);
criterion_main!(benches);
