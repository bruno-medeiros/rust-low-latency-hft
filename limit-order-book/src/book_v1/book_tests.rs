use crate::book_tests_common;
use crate::book_v1::book::LimitOrderBookV1;
use crate::book_v1::price_level::PriceLevel;
use crate::event::EventKind;
use crate::order::Order;
use crate::types::Side;
use std::collections::HashMap;

#[test]
fn reject_zero_quantity() {
    book_tests_common::reject_zero_quantity(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn reject_zero_price() {
    book_tests_common::reject_zero_price(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn add_limit_order_rests_in_book() {
    book_tests_common::add_limit_order_rests_in_book(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn add_and_cancel() {
    book_tests_common::add_and_cancel(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn cancel_unknown_order() {
    book_tests_common::cancel_unknown_order(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn cancel_one_of_many_at_same_price() {
    book_tests_common::cancel_one_of_many_at_same_price(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn reject_duplicate_id() {
    book_tests_common::reject_duplicate_id(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn event_sequences_are_monotonic() {
    book_tests_common::event_sequences_are_monotonic(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn best_bid_best_ask() {
    book_tests_common::best_bid_best_ask(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn limit_order_full_match() {
    book_tests_common::limit_order_full_match(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn limit_order_partial_match_passive_remains() {
    book_tests_common::limit_order_partial_match_passive_remains(LimitOrderBookV1::new((
        0, 10_000,
    )));
}

#[test]
fn market_order_full_fill() {
    book_tests_common::market_order_full_fill(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn market_order_partial_fill_exhausts_book() {
    book_tests_common::market_order_partial_fill_exhausts_book_and_emits_cancel(
        LimitOrderBookV1::new((0, 10_000)),
    );
}

#[test]
fn fifo_priority() {
    book_tests_common::fifo_priority(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn multi_level_sweep() {
    book_tests_common::multi_level_sweep(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn no_match_when_prices_dont_cross() {
    book_tests_common::no_match_when_prices_dont_cross(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn sell_side_matching_hits_best_bid_first() {
    book_tests_common::sell_side_matching_hits_best_bid_first(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn order_preserves_original_qty_after_partial_fill() {
    book_tests_common::order_preserves_original_qty_after_partial_fill(LimitOrderBookV1::new((
        0, 10_000,
    )));
}

#[test]
fn market_order_rejects_duplicate_id() {
    book_tests_common::market_order_rejects_duplicate_id(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn market_order_emits_accepted_event() {
    book_tests_common::market_order_emits_accepted_event(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn cancel_front_preserves_fifo_for_remaining() {
    book_tests_common::cancel_front_preserves_fifo_for_remaining(LimitOrderBookV1::new((
        0, 10_000,
    )));
}

#[test]
fn sweep_multiple_orders_at_same_level() {
    book_tests_common::sweep_multiple_orders_at_same_level(LimitOrderBookV1::new((0, 10_000)));
}

#[test]
fn fulfill_in_price_level_fills_multiple_passive_orders() {
    let mut price_level = PriceLevel::new(50);
    let mut orders = HashMap::new();
    let mut events = Vec::new();
    let mut next_seq = 0u64;
    let aggressor_id = 100;

    for id in 1..=3u64 {
        price_level.push(id, 10);
        orders.insert(
            id,
            Order {
                id,
                side: Side::Sell,
                price: 50,
                qty: 10,
                remaining_qty: 10,
                sequence: id,
            },
        );
    }

    let mut remaining = 25u64;

    LimitOrderBookV1::fulfill_in_price_level(
        &mut price_level,
        &mut orders,
        &mut events,
        &mut next_seq,
        aggressor_id,
        &mut remaining,
    );

    assert_eq!(
        remaining, 0,
        "aggressor had 25 qty vs 30 available — should be fully satisfied"
    );

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(
        fills.len(),
        3,
        "expected Fill events for all three passive orders"
    );

    assert!(matches!(
        fills[0].kind,
        EventKind::Fill {
            passive_order_id: 1,
            aggressor_order_id: 100,
            price: 50,
            qty: 10,
        }
    ));
    assert!(matches!(
        fills[1].kind,
        EventKind::Fill {
            passive_order_id: 2,
            aggressor_order_id: 100,
            price: 50,
            qty: 10,
        }
    ));
    assert!(matches!(
        fills[2].kind,
        EventKind::Fill {
            passive_order_id: 3,
            aggressor_order_id: 100,
            price: 50,
            qty: 5,
        }
    ));

    assert!(
        !orders.contains_key(&1),
        "order 1 fully filled — should be removed"
    );
    assert!(
        !orders.contains_key(&2),
        "order 2 fully filled — should be removed"
    );
    let o3 = orders
        .get(&3)
        .expect("order 3 partially filled — should remain");
    assert_eq!(o3.remaining_qty, 5);

    assert!(!price_level.is_empty());
    assert_eq!(price_level.total_qty, 5);
}

#[test]
fn fulfill_in_price_level_exhausts_all_passive_orders() {
    let mut price_level = PriceLevel::new(200);
    let mut orders = HashMap::new();
    let mut events = Vec::new();
    let mut next_seq = 0u64;
    let aggressor_id = 100;

    for id in 1..=3u64 {
        price_level.push(id, 5);
        orders.insert(
            id,
            Order {
                id,
                side: Side::Buy,
                price: 200,
                qty: 5,
                remaining_qty: 5,
                sequence: id,
            },
        );
    }

    let mut remaining = 20u64;

    LimitOrderBookV1::fulfill_in_price_level(
        &mut price_level,
        &mut orders,
        &mut events,
        &mut next_seq,
        aggressor_id,
        &mut remaining,
    );

    assert_eq!(
        remaining, 5,
        "15 available vs 20 requested — 5 should remain"
    );

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 3, "all three passive orders should be filled");

    let filled: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Filled { .. }))
        .collect();
    assert_eq!(
        filled.len(),
        3,
        "all three passive orders should be fully filled"
    );

    assert!(
        orders.is_empty(),
        "all passive orders consumed — map should be empty"
    );
    assert!(price_level.is_empty());
    assert_eq!(price_level.total_qty, 0);
}
