use crate::LimitOrderBook;
use crate::event::{BookEvent, BookEventKind, RejectReason};
use crate::types::Side;

pub fn reject_zero_quantity(mut book: impl LimitOrderBook) {
    let events = book.add_limit_order_vec(1, Side::Buy, 100, 0);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(1, RejectReason::InvalidQuantity)
    );
    assert_eq!(book.order_count(), 0);
}

pub fn reject_zero_price(mut book: impl LimitOrderBook) {
    let events = book.add_limit_order_vec(1, Side::Buy, 0, 10);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(1, RejectReason::InvalidPrice)
    );
}

pub fn add_limit_order_rests_in_book(mut book: impl LimitOrderBook) {
    let events = book.add_limit_order_vec(1, Side::Buy, 100, 10);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, BookEventKind::Accepted { order_id: 1 });
    assert_eq!(book.best_bid(), Some((100, 10)));
    assert_eq!(book.best_ask(), None);
    assert_eq!(book.order_count(), 1);
}

pub fn add_and_cancel(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Buy, 100, 10);

    let events = book.cancel_order_vec(1);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, BookEventKind::cancelled(1, 10));
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.best_ask(), None);
    assert_eq!(book.order_count(), 0);
}

pub fn cancel_unknown_order(mut book: impl LimitOrderBook) {
    let events = book.cancel_order_vec(999);

    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(999, RejectReason::UnknownOrder)
    );
}

pub fn cancel_one_of_many_at_same_price(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);
    book.add_limit_order_vec(2, Side::Sell, 100, 20);
    book.add_limit_order_vec(3, Side::Sell, 100, 30);

    let events = book.cancel_order_vec(2);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, BookEventKind::cancelled(2, 20));

    assert_eq!(book.order_count(), 2);
    assert_eq!(book.best_ask(), Some((100, 40)));
    assert!(book.order(1).is_some());
    assert!(book.order(2).is_none());
    assert!(book.order(3).is_some());
}

pub fn reduce_unknown_order(mut book: impl LimitOrderBook) {
    let events = book.reduce_order_vec(999, 1);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(999, RejectReason::UnknownOrder)
    );
}

pub fn reduce_order_rejects_zero_quantity(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);
    let events = book.reduce_order_vec(1, 0);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(1, RejectReason::InvalidQuantity)
    );
    assert_eq!(book.best_ask(), Some((100, 10)));
}

pub fn reduce_order_partial_reduces_resting_qty(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);
    let events = book.reduce_order_vec(1, 4);
    assert!(events.is_empty(), "partial reduce should not emit events");
    assert_eq!(book.order_count(), 1);
    assert_eq!(book.best_ask(), Some((100, 6)));
    let order = book.order(1).expect("order remains after partial reduce");
    assert_eq!(order.qty, 10);
    assert_eq!(order.remaining_qty, 6);
}

pub fn reduce_order_full_reduction_removes_order(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);
    let events = book.reduce_order_vec(1, 10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, BookEventKind::cancelled(1, 10));
    assert_eq!(book.order_count(), 0);
    assert_eq!(book.best_ask(), None);
    assert!(book.order(1).is_none());
}

pub fn reject_duplicate_id(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Buy, 100, 10);
    let events = book.add_limit_order_vec(1, Side::Buy, 101, 5);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(1, RejectReason::DuplicateOrderId)
    );
    assert_eq!(book.order_count(), 1);
}

pub fn event_sequences_are_monotonic(mut book: impl LimitOrderBook) {
    let mut all: Vec<BookEvent> = Vec::new();
    all.extend(book.add_limit_order_vec(1, Side::Sell, 100, 10));
    all.extend(book.add_limit_order_vec(2, Side::Buy, 100, 10));
    all.extend(book.cancel_order_vec(999));

    for window in all.windows(2) {
        assert!(
            window[0].sequence < window[1].sequence,
            "sequences must be strictly increasing: {} vs {}",
            window[0].sequence,
            window[1].sequence
        );
    }
}

pub fn best_bid_best_ask(mut book: impl LimitOrderBook) {
    // Test best bid best ask over more complex cases
    book.add_limit_order_vec(1, Side::Buy, 100, 10);
    book.add_limit_order_vec(2, Side::Buy, 110, 20);
    book.add_limit_order_vec(3, Side::Sell, 120, 20);
    book.add_limit_order_vec(4, Side::Sell, 130, 10);

    assert_eq!(book.best_bid(), Some((110, 20)));
    assert_eq!(book.best_ask(), Some((120, 20)));

    assert_eq!(book.depth(Side::Buy, 3), vec![(110, 20), (100, 10)]);
    assert_eq!(book.depth(Side::Sell, 3), vec![(120, 20), (130, 10)]);

    book.cancel_order_vec(2);
    book.cancel_order_vec(3);

    assert_eq!(book.best_bid(), Some((100, 10)));
    assert_eq!(book.best_ask(), Some((130, 10)));

    assert_eq!(book.depth(Side::Buy, 3), vec![(100, 10)]);
    assert_eq!(book.depth(Side::Sell, 3), vec![(130, 10)]);
}

pub fn limit_order_full_match(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);

    let events = book.add_limit_order_vec(2, Side::Buy, 100, 10);
    assert_eq!(events.len(), 4);
    assert_eq!(events[0].kind, BookEventKind::Accepted { order_id: 2 });
    assert_eq!(
        events[1].kind,
        BookEventKind::Fill {
            aggressor_order_id: 2,
            passive_order_id: 1,
            price: 100,
            qty: 10
        }
    );
    assert_eq!(events[2].kind, BookEventKind::Filled { order_id: 1 });
    assert_eq!(events[3].kind, BookEventKind::Filled { order_id: 2 });
    assert!(book.order(1).is_none());
    assert!(book.order(2).is_none());
    assert_eq!(book.order_count(), 0);
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.best_ask(), None);
}

pub fn limit_order_partial_match_passive_remains(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);

    let events = book.add_limit_order_vec(2, Side::Buy, 100, 5);
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[1].kind,
        BookEventKind::Fill {
            aggressor_order_id: 2,
            passive_order_id: 1,
            price: 100,
            qty: 5
        }
    );
    assert_eq!(events[2].kind, BookEventKind::Filled { order_id: 2 });

    assert_eq!(book.best_ask(), Some((100, 5)));
    assert_eq!(book.best_bid(), None);
    // Incoming imit rests after partial fill:
    assert_eq!(book.order(1).unwrap().remaining_qty, 5);
    assert_eq!(book.order_count(), 1);
}

pub fn market_order_full_fill(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 5);
    book.add_limit_order_vec(2, Side::Sell, 101, 5);
    assert_eq!(book.order_count(), 2);
    assert!(book.order(1).is_some());

    let events = book.add_market_order_vec(3, Side::Buy, 8);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, BookEventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 2);
    assert!(matches!(
        fills[0].kind,
        BookEventKind::Fill {
            price: 100,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        BookEventKind::Fill {
            price: 101,
            qty: 3,
            ..
        }
    ));
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, BookEventKind::Filled { order_id: 3 }))
    );
    assert_eq!(book.best_ask(), Some((101, 2)));
}

pub fn market_order_partial_fill_exhausts_book_and_emits_cancel(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 5);

    let events = book.add_market_order_vec(2, Side::Buy, 10);
    assert!(events
        .iter()
        .any(|e| e.kind == BookEventKind::cancelled(2, 5)));
    assert_eq!(book.best_ask(), None);
}

pub fn market_order_rejects_duplicate_id(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);

    let events = book.add_market_order_vec(1, Side::Buy, 5);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind,
        BookEventKind::rejected(1, RejectReason::DuplicateOrderId)
    );
    assert_eq!(book.best_ask(), Some((100, 10)));
}

pub fn market_order_emits_accepted_event(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);

    let events = book.add_market_order_vec(2, Side::Buy, 5);
    assert!(matches!(
        events[0].kind,
        BookEventKind::Accepted { order_id: 2 }
    ));
}

// ==== More matching tests

pub fn fifo_priority(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);
    book.add_limit_order_vec(2, Side::Sell, 100, 10);

    let events = book.add_limit_order_vec(3, Side::Buy, 100, 10);
    assert!(matches!(
        events[1].kind,
        BookEventKind::Fill {
            passive_order_id: 1,
            qty: 10,
            ..
        }
    ));
    assert_eq!(events[2].kind, BookEventKind::Filled { order_id: 1 });
    assert_eq!(book.order(2).unwrap().remaining_qty, 10);
    assert_eq!(book.best_ask(), Some((100, 10)));
}

pub fn cancel_front_preserves_fifo_for_remaining(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 10);
    book.add_limit_order_vec(2, Side::Sell, 100, 10);
    book.add_limit_order_vec(3, Side::Sell, 100, 10);

    book.cancel_order_vec(1);

    let events = book.add_limit_order_vec(4, Side::Buy, 100, 10);
    assert!(matches!(
        events[1].kind,
        BookEventKind::Fill {
            passive_order_id: 2,
            qty: 10,
            ..
        }
    ));
    assert_eq!(book.order(3).unwrap().remaining_qty, 10);
}

pub fn multi_level_sweep(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 5);
    book.add_limit_order_vec(2, Side::Sell, 101, 5);
    book.add_limit_order_vec(3, Side::Sell, 102, 5);

    let events = book.add_limit_order_vec(4, Side::Buy, 102, 12);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, BookEventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 3);
    assert!(matches!(
        fills[0].kind,
        BookEventKind::Fill {
            aggressor_order_id: 4,
            passive_order_id: 1,
            price: 100,
            qty: 5,
        }
    ));
    assert!(matches!(
        fills[1].kind,
        BookEventKind::Fill {
            aggressor_order_id: 4,
            passive_order_id: 2,
            price: 101,
            qty: 5,
        }
    ));
    assert!(matches!(
        fills[2].kind,
        BookEventKind::Fill {
            aggressor_order_id: 4,
            passive_order_id: 3,
            price: 102,
            qty: 2,
        }
    ));

    assert_eq!(book.best_ask(), Some((102, 3)));
    assert_eq!(book.order_count(), 1);
}

pub fn no_match_when_prices_dont_cross(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 101, 10);
    book.add_limit_order_vec(2, Side::Buy, 99, 10);

    assert_eq!(book.best_bid(), Some((99, 10)));
    assert_eq!(book.best_ask(), Some((101, 10)));
    assert_eq!(book.order_count(), 2);
    assert_eq!(book.spread(), Some(2));
}

pub fn sell_side_matching_hits_best_bid_first(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Buy, 100, 10);
    book.add_limit_order_vec(2, Side::Buy, 99, 10);

    let events = book.add_limit_order_vec(3, Side::Sell, 99, 15);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, BookEventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 2);
    assert!(matches!(
        fills[0].kind,
        BookEventKind::Fill {
            passive_order_id: 1,
            price: 100,
            qty: 10,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        BookEventKind::Fill {
            passive_order_id: 2,
            price: 99,
            qty: 5,
            ..
        }
    ));
    assert_eq!(book.best_bid(), Some((99, 5)));
    assert_eq!(book.best_ask(), None);
}

pub fn order_preserves_original_qty_after_partial_fill(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 3);
    book.add_limit_order_vec(2, Side::Buy, 100, 10);

    let order = book.order(2).unwrap();
    assert_eq!(order.qty, 10, "original qty must be preserved");
    assert_eq!(order.remaining_qty, 7);
}

pub fn sweep_multiple_orders_at_same_level(mut book: impl LimitOrderBook) {
    book.add_limit_order_vec(1, Side::Sell, 100, 5);
    book.add_limit_order_vec(2, Side::Sell, 100, 5);
    book.add_limit_order_vec(3, Side::Sell, 100, 5);

    let events = book.add_limit_order_vec(4, Side::Buy, 100, 12);

    let fills: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, BookEventKind::Fill { .. }))
        .collect();
    assert_eq!(fills.len(), 3);
    assert!(matches!(
        fills[0].kind,
        BookEventKind::Fill {
            passive_order_id: 1,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[1].kind,
        BookEventKind::Fill {
            passive_order_id: 2,
            qty: 5,
            ..
        }
    ));
    assert!(matches!(
        fills[2].kind,
        BookEventKind::Fill {
            passive_order_id: 3,
            qty: 2,
            ..
        }
    ));
    assert_eq!(book.best_ask(), Some((100, 3)));
    assert_eq!(book.best_bid(), None);
    assert_eq!(book.order_count(), 1);
}
