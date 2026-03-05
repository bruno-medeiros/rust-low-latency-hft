use std::collections::{BTreeMap, HashMap};

use crate::event::{Event, EventKind, RejectReason};
use crate::order::Order;
use crate::price_level::PriceLevel;
use crate::types::{OrderId, Price, Qty, Side};

#[derive(Debug)]
pub struct LimitOrderBook {
    bids: BTreeMap<Price, PriceLevel>,
    asks: BTreeMap<Price, PriceLevel>,
    orders: HashMap<OrderId, Order>,
    next_seq: u64,
}

fn emit(seq: &mut u64, kind: EventKind) -> Event {
    let s = *seq;
    *seq += 1;
    Event { sequence: s, kind }
}

impl LimitOrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            next_seq: 0,
        }
    }

    fn emit(&mut self, kind: EventKind) -> Event {
        let s = self.next_seq;
        self.next_seq += 1;
        Event { sequence: s, kind }
    }

    // -- queries -----------------------------------------------------------

    pub fn best_bid(&self) -> Option<(Price, Qty)> {
        self.bids
            .last_key_value()
            .map(|(&p, lvl)| (p, lvl.total_qty))
    }

    pub fn best_ask(&self) -> Option<(Price, Qty)> {
        self.asks
            .first_key_value()
            .map(|(&p, lvl)| (p, lvl.total_qty))
    }

    pub fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    pub fn order(&self, id: OrderId) -> Option<&Order> {
        self.orders.get(&id)
    }

    pub fn order_count(&self) -> usize {
        self.orders.len()
    }

    // -- commands ----------------------------------------------------------

    pub fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
    ) -> Vec<Event> {
        if let Some(value) = self.validate_order_qty(order_id, qty) {
            return value;
        }
        if price == 0 {
            let err_event = emit(
                &mut self.next_seq,
                EventKind::Rejected {
                    order_id,
                    reason: RejectReason::InvalidPrice,
                },
            );
            return vec![err_event];
        }

        if self.orders.contains_key(&order_id) {
            let err_event = self.emit(EventKind::Rejected {
                order_id,
                reason: RejectReason::DuplicateOrderId,
            });
            return vec![err_event];
        }
        let mut events = Vec::new();

        events.push(emit(&mut self.next_seq, EventKind::Accepted { order_id }));

        self.orders.insert(
            order_id,
            Order {
                id: order_id,
                side,
                price,
                qty,
                remaining_qty: qty,
                sequence: self.next_seq,
            },
        );

        match side {
            Side::Buy => {
                self.bids.entry(price).or_default().push(order_id, qty);
            }
            Side::Sell => {
                self.asks.entry(price).or_default().push(order_id, qty);
            }
        }

        // TODO: try_match

        events
    }

    fn validate_order_qty(&mut self, id: OrderId, qty: Qty) -> Option<Vec<Event>> {
        if qty == 0 {
            let err_event = emit(
                &mut self.next_seq,
                EventKind::Rejected {
                    order_id: id,
                    reason: RejectReason::InvalidQuantity,
                },
            );
            return Some(vec![err_event]);
        }

        None
    }

    pub fn add_market_order(&mut self, id: OrderId, side: Side, qty: Qty) -> Vec<Event> {
        if let Some(value) = self.validate_order_qty(id, qty) {
            return value;
        }
        let mut events = Vec::new();

        events
    }

    pub fn cancel_order(&mut self, id: OrderId) -> Vec<Event> {
        match self.orders.remove(&id) {
            None => {
                let event = emit(
                    &mut self.next_seq,
                    EventKind::Rejected {
                        order_id: id,
                        reason: RejectReason::UnknownOrder,
                    },
                );
                vec![event]
            }
            Some(order) => {
                let mut events = Vec::new();

                match order.side {
                    Side::Buy => {
                        self.bids.remove(&order.price);
                    }
                    Side::Sell => {
                        self.asks.remove(&order.price);
                    }
                }

                events.push(emit(
                    &mut self.next_seq,
                    EventKind::Cancelled {
                        order_id: id,
                        remaining_qty: order.remaining_qty,
                    },
                ));

                events
            }
        }
    }
}

impl Default for LimitOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_limit_order_rests_in_book() {
        let mut book = LimitOrderBook::new();
        let events = book.add_limit_order(1, Side::Buy, 100, 10);

        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].kind,
            EventKind::Accepted { order_id: 1 }
        ));
        assert_eq!(book.best_bid(), Some((100, 10)));
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.order_count(), 1);
    }

    #[test]
    fn add_and_cancel() {
        let mut book = LimitOrderBook::new();
        book.add_limit_order(1, Side::Buy, 100, 10);

        let events = book.cancel_order(1);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].kind,
            EventKind::Cancelled {
                order_id: 1,
                remaining_qty: 10
            }
        ));
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.order_count(), 0);
    }

    #[test]
    fn cancel_unknown_order() {
        let mut book = LimitOrderBook::new();
        let events = book.cancel_order(999);

        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].kind,
            EventKind::Rejected {
                order_id: 999,
                reason: RejectReason::UnknownOrder
            }
        ));
    }

    #[test]
    fn limit_order_full_match() {
        let mut book = LimitOrderBook::new();
        book.add_limit_order(1, Side::Sell, 100, 10);

        let events = book.add_limit_order(2, Side::Buy, 100, 10);
        assert_eq!(events.len(), 4);
        assert!(matches!(
            events[0].kind,
            EventKind::Accepted { order_id: 2 }
        ));
        assert!(matches!(
            events[1].kind,
            EventKind::Fill {
                aggressor_order_id: 2,
                passive_order_id: 1,
                price: 100,
                qty: 10
            }
        ));
        assert!(matches!(events[2].kind, EventKind::Filled { order_id: 1 }));
        assert!(matches!(events[3].kind, EventKind::Filled { order_id: 2 }));
        assert_eq!(book.order_count(), 0);
        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn limit_order_partial_match_passive_remains() {
        let mut book = LimitOrderBook::new();
        book.add_limit_order(1, Side::Sell, 100, 10);

        let events = book.add_limit_order(2, Side::Buy, 100, 5);
        assert_eq!(events.len(), 3);
        assert!(matches!(
            events[1].kind,
            EventKind::Fill {
                aggressor_order_id: 2,
                passive_order_id: 1,
                price: 100,
                qty: 5
            }
        ));
        assert!(matches!(events[2].kind, EventKind::Filled { order_id: 2 }));

        assert_eq!(book.best_ask(), Some((100, 5)));
        assert_eq!(book.order(1).unwrap().remaining_qty, 5);
        assert_eq!(book.order_count(), 1);
    }

    #[test]
    fn incoming_limit_rests_after_partial_fill() {
        let mut book = LimitOrderBook::new();
        book.add_limit_order(1, Side::Sell, 100, 5);
        book.add_limit_order(2, Side::Buy, 100, 10);

        assert_eq!(book.best_bid(), Some((100, 5)));
        assert_eq!(book.best_ask(), None);
        assert_eq!(book.order(2).unwrap().remaining_qty, 5);
    }

    #[test]
    fn market_order_full_fill() {
        let mut book = LimitOrderBook::new();
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
        let mut book = LimitOrderBook::new();
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
        let mut book = LimitOrderBook::new();
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
        assert!(matches!(events[2].kind, EventKind::Filled { order_id: 1 }));
        assert_eq!(book.order(2).unwrap().remaining_qty, 10);
        assert_eq!(book.best_ask(), Some((100, 10)));
    }

    #[test]
    fn multi_level_sweep() {
        let mut book = LimitOrderBook::new();
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
        let mut book = LimitOrderBook::new();
        book.add_limit_order(1, Side::Sell, 101, 10);
        book.add_limit_order(2, Side::Buy, 99, 10);

        assert_eq!(book.best_bid(), Some((99, 10)));
        assert_eq!(book.best_ask(), Some((101, 10)));
        assert_eq!(book.order_count(), 2);
        assert_eq!(book.spread(), Some(2));
    }

    #[test]
    fn sell_side_matching_hits_best_bid_first() {
        let mut book = LimitOrderBook::new();
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
    fn reject_zero_quantity() {
        let mut book = LimitOrderBook::new();
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
        let mut book = LimitOrderBook::new();
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
    fn reject_duplicate_id() {
        let mut book = LimitOrderBook::new();
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
        let mut book = LimitOrderBook::new();
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
}
