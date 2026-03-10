use crate::book_v1::book::LimitOrderBookV1;
use crate::book_v1::price_level::PriceLevel;
use crate::event::{Event, EventKind, RejectReason};
use crate::order::Order;
use crate::types::Side;
use std::collections::HashMap;

#[test]
fn reject_zero_quantity() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    let events = book.add_limit_order(1, Side::Buy, 100, 0);
    assert!(matches!(
        events[0].kind,
        EventKind::Rejected {
            reason: RejectReason::InvalidQuantity,
            ..
        }
    ));
    assert_eq!(book.order_count(), 0);
}

#[test]
fn reject_zero_price() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    let events = book.add_limit_order(1, Side::Buy, 0, 10);
    assert!(matches!(
        events[0].kind,
        EventKind::Rejected {
            reason: RejectReason::InvalidPrice,
            ..
        }
    ));
}

#[test]
fn add_limit_order_rests_in_book() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    let events = book.add_limit_order(1, Side::Buy, 100, 10);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, EventKind::Accepted { order_id: 1 });
    assert_eq!(book.best_bid(), Some((100, 10)));
    assert_eq!(book.best_ask(), None);
    assert_eq!(book.order_count(), 1);
}

#[test]
fn add_and_cancel() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Buy, 100, 10);

    let events = book.cancel_order(1);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind,
        EventKind::Cancelled {
            order_id: 1,
            remaining_qty: 10
        }
    );
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.order_count(), 0);
}

#[test]
fn cancel_unknown_order() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    let events = book.cancel_order(999);

    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind,
        EventKind::Rejected {
            order_id: 999,
            reason: RejectReason::UnknownOrder
        }
    );
}

#[test]
fn cancel_one_of_many_at_same_price() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);
    book.add_limit_order(2, Side::Sell, 100, 20);
    book.add_limit_order(3, Side::Sell, 100, 30);

    let events = book.cancel_order(2);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].kind,
        EventKind::Cancelled {
            order_id: 2,
            remaining_qty: 20
        }
    ));

    assert_eq!(book.order_count(), 2);
    assert_eq!(book.best_ask(), Some((100, 40)));
    assert!(book.order(1).is_some());
    assert!(book.order(2).is_none());
    assert!(book.order(3).is_some());

    let events = book.add_limit_order(4, Side::Buy, 100, 15);
    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 2);
    assert!(matches!(
        fills[0].kind,
        EventKind::Fill {
            passive_order_id: 1,
            qty: 10,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        EventKind::Fill {
            passive_order_id: 3,
            qty: 5,
            ..
        }
    ));
    assert_eq!(book.best_ask(), Some((100, 25)));
}

#[test]
fn reject_duplicate_id() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Buy, 100, 10);
    let events = book.add_limit_order(1, Side::Buy, 101, 5);
    assert!(matches!(
        events[0].kind,
        EventKind::Rejected {
            reason: RejectReason::DuplicateOrderId,
            ..
        }
    ));
    assert_eq!(book.order_count(), 1);
}

#[test]
fn event_sequences_are_monotonic() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    let mut all: Vec<Event> = Vec::new();
    all.extend(book.add_limit_order(1, Side::Sell, 100, 10));
    all.extend(book.add_limit_order(2, Side::Buy, 100, 10));
    all.extend(book.cancel_order(999));

    for w in all.windows(2) {
        assert!(
            w[0].sequence < w[1].sequence,
            "sequences must be strictly increasing: {} vs {}",
            w[0].sequence,
            w[1].sequence
        );
    }
}

#[test]
fn best_bid_best_ask() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Buy, 100, 10);
    book.add_limit_order(2, Side::Buy, 110, 20);
    book.add_limit_order(3, Side::Sell, 120, 20);
    book.add_limit_order(4, Side::Sell, 130, 10);

    assert_eq!(book.best_bid(), Some((110, 20)));
    assert_eq!(book.best_ask(), Some((120, 20)));

    assert_eq!(book.depth(Side::Buy, 3), vec![(110, 20), (100, 10)]);
    assert_eq!(book.depth(Side::Sell, 3), vec![(120, 20), (130, 10)]);

    // Test after cancellation
    book.cancel_order(2);
    book.cancel_order(3);

    assert_eq!(book.best_bid(), Some((100, 10)));
    assert_eq!(book.best_ask(), Some((130, 10)));

    assert_eq!(book.depth(Side::Buy, 3), vec![(100, 10)]);
    assert_eq!(book.depth(Side::Sell, 3), vec![(130, 10)]);
}

// ------------ Order Matching  ------------

#[test]
fn limit_order_full_match() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);

    let events = book.add_limit_order(2, Side::Buy, 100, 10);
    assert_eq!(events.len(), 4);
    assert_eq!(events[0].kind, EventKind::Accepted { order_id: 2 });
    assert_eq!(
        events[1].kind,
        EventKind::Fill {
            aggressor_order_id: 2,
            passive_order_id: 1,
            price: 100,
            qty: 10
        }
    );
    assert_eq!(events[2].kind, EventKind::Filled { order_id: 1 });
    assert_eq!(events[3].kind, EventKind::Filled { order_id: 2 });
    assert_eq!(book.orders, HashMap::new());
    assert_eq!(book.order_count(), 0);
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.best_ask(), None);
}

#[test]
fn limit_order_partial_match_passive_remains() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);

    let events = book.add_limit_order(2, Side::Buy, 100, 5);
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[1].kind,
        EventKind::Fill {
            aggressor_order_id: 2,
            passive_order_id: 1,
            price: 100,
            qty: 5
        }
    );
    assert_eq!(events[2].kind, EventKind::Filled { order_id: 2 });

    assert_eq!(book.best_ask(), Some((100, 5)));
    assert_eq!(book.order(1).unwrap().remaining_qty, 5);
    assert_eq!(book.order_count(), 1);
}

#[test]
fn incoming_limit_rests_after_partial_fill() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 5);
    book.add_limit_order(2, Side::Buy, 100, 10);

    assert_eq!(book.best_bid(), Some((100, 5)));
    assert_eq!(book.best_ask(), None);
    assert_eq!(book.order(2).unwrap().remaining_qty, 5);
}

#[test]
fn market_order_full_fill() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 5);
    book.add_limit_order(2, Side::Sell, 101, 5);

    let events = book.add_market_order(3, Side::Buy, 8);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 2);
    assert!(matches!(
        fills[0].kind,
        EventKind::Fill {
            price: 100,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        EventKind::Fill {
            price: 101,
            qty: 3,
            ..
        }
    ));
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Filled { order_id: 3 }))
    );
    assert_eq!(book.best_ask(), Some((101, 2)));
}

#[test]
fn market_order_partial_fill_exhausts_book() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 5);

    let events = book.add_market_order(2, Side::Buy, 10);
    assert!(events.iter().any(|e| matches!(
        e.kind,
        EventKind::Cancelled {
            order_id: 2,
            remaining_qty: 5
        }
    )));
    assert_eq!(book.best_ask(), None);
}

#[test]
fn fifo_priority() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);
    book.add_limit_order(2, Side::Sell, 100, 10);

    let events = book.add_limit_order(3, Side::Buy, 100, 10);
    assert!(matches!(
        events[1].kind,
        EventKind::Fill {
            passive_order_id: 1,
            qty: 10,
            ..
        }
    ));
    assert_eq!(events[2].kind, EventKind::Filled { order_id: 1 });
    assert_eq!(book.order(2).unwrap().remaining_qty, 10);
    assert_eq!(book.best_ask(), Some((100, 10)));
}

#[test]
fn multi_level_sweep() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 5);
    book.add_limit_order(2, Side::Sell, 101, 5);
    book.add_limit_order(3, Side::Sell, 102, 5);

    let events = book.add_limit_order(4, Side::Buy, 102, 12);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 3);
    assert!(matches!(
        fills[0].kind,
        EventKind::Fill {
            passive_order_id: 1,
            price: 100,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        EventKind::Fill {
            passive_order_id: 2,
            price: 101,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[2].kind,
        EventKind::Fill {
            passive_order_id: 3,
            price: 102,
            qty: 2,
            ..
        }
    ));

    assert_eq!(book.best_ask(), Some((102, 3)));
    assert_eq!(book.order_count(), 1);
}

#[test]
fn no_match_when_prices_dont_cross() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 101, 10);
    book.add_limit_order(2, Side::Buy, 99, 10);

    assert_eq!(book.best_bid(), Some((99, 10)));
    assert_eq!(book.best_ask(), Some((101, 10)));
    assert_eq!(book.order_count(), 2);
    assert_eq!(
        match (book.best_bid(), book.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask.saturating_sub(bid)),
            _ => None,
        },
        Some(2)
    );
}

#[test]
fn sell_side_matching_hits_best_bid_first() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Buy, 100, 10);
    book.add_limit_order(2, Side::Buy, 99, 10);

    let events = book.add_limit_order(3, Side::Sell, 99, 15);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 2);
    assert!(matches!(
        fills[0].kind,
        EventKind::Fill {
            passive_order_id: 1,
            price: 100,
            qty: 10,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        EventKind::Fill {
            passive_order_id: 2,
            price: 99,
            qty: 5,
            ..
        }
    ));
    assert_eq!(book.best_bid(), Some((99, 5)));
    assert_eq!(book.best_ask(), None);
}

#[test]
fn order_preserves_original_qty_after_partial_fill() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 3);
    book.add_limit_order(2, Side::Buy, 100, 10);

    let order = book.order(2).unwrap();
    assert_eq!(order.qty, 10, "original qty must be preserved");
    assert_eq!(order.remaining_qty, 7);
}

#[test]
fn market_order_rejects_duplicate_id() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);

    let events = book.add_market_order(1, Side::Buy, 5);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].kind,
        EventKind::Rejected {
            order_id: 1,
            reason: RejectReason::DuplicateOrderId
        }
    ));
    assert_eq!(book.best_ask(), Some((100, 10)));
}

#[test]
fn market_order_emits_accepted_event() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);

    let events = book.add_market_order(2, Side::Buy, 5);
    assert!(matches!(
        events[0].kind,
        EventKind::Accepted { order_id: 2 }
    ));
}

#[test]
fn cancel_front_preserves_fifo_for_remaining() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 10);
    book.add_limit_order(2, Side::Sell, 100, 10);
    book.add_limit_order(3, Side::Sell, 100, 10);

    book.cancel_order(1);

    let events = book.add_limit_order(4, Side::Buy, 100, 10);
    assert!(matches!(
        events[1].kind,
        EventKind::Fill {
            passive_order_id: 2,
            qty: 10,
            ..
        }
    ));
    assert_eq!(book.order(3).unwrap().remaining_qty, 10);
}

#[test]
fn sweep_multiple_orders_at_same_level() {
    let mut book = LimitOrderBookV1::new((0, 10_000));
    book.add_limit_order(1, Side::Sell, 100, 5);
    book.add_limit_order(2, Side::Sell, 100, 5);
    book.add_limit_order(3, Side::Sell, 100, 5);

    let events = book.add_limit_order(4, Side::Buy, 100, 12);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 3);
    assert!(matches!(
        fills[0].kind,
        EventKind::Fill {
            passive_order_id: 1,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        EventKind::Fill {
            passive_order_id: 2,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[2].kind,
        EventKind::Fill {
            passive_order_id: 3,
            qty: 2,
            ..
        }
    ));
    assert_eq!(book.best_ask(), Some((100, 3)));
    assert_eq!(book.order_count(), 1);
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
