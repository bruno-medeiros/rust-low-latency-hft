use crate::LimitOrderBook;
use crate::book_v0::price_level::PriceLevel;
use crate::event::{Event, EventKind, EventSink, RejectReason};
use crate::order::Order;
use crate::types::{OrderId, Price, Qty, Side};
use std::collections::btree_map::Iter;
use std::collections::{BTreeMap, HashMap};
use std::iter::Rev;

#[derive(Debug)]
pub struct LimitOrderBookV0 {
    pub(crate) bids: BTreeMap<Price, PriceLevel>,
    pub(crate) asks: BTreeMap<Price, PriceLevel>,
    pub(crate) orders: HashMap<OrderId, Order>,
    pub(crate) next_seq: u64,
}

//noinspection DuplicatedCode
fn emit(events: &mut impl EventSink, seq: &mut u64, kind: EventKind) {
    let s = *seq;
    *seq += 1;
    events.push(Event { sequence: s, kind })
}

//noinspection DuplicatedCode
impl LimitOrderBookV0 {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            next_seq: 0,
        }
    }

    fn emit(&mut self, events: &mut impl EventSink, kind: EventKind) {
        emit(events, &mut self.next_seq, kind)
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

    pub fn order_count(&self) -> u64 {
        self.orders.len() as u64
    }

    pub fn depth(&self, side: Side, levels: u64) -> Vec<(Price, Qty)> {
        match side {
            Side::Buy => self
                .bids_iter()
                .take(levels as usize)
                .map(|(&p, lvl)| (p, lvl.total_qty))
                .collect(),
            Side::Sell => self
                .asks_iter()
                .take(levels as usize)
                .map(|(&p, lvl)| (p, lvl.total_qty))
                .collect(),
        }
    }

    fn asks_iter(&self) -> Iter<'_, Price, PriceLevel> {
        self.asks.iter()
    }

    fn bids_iter(&self) -> Rev<Iter<'_, Price, PriceLevel>> {
        self.bids.iter().rev()
    }
    // -- commands ----------------------------------------------------------

    pub fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
        events: &mut impl EventSink,
    ) {
        if self.validate_order_qty(order_id, qty, events) {
            return;
        }
        if price == 0 {
            self.emit(
                events,
                EventKind::rejected(order_id, RejectReason::InvalidPrice),
            );
            return;
        }

        if self.orders.contains_key(&order_id) {
            self.emit(
                events,
                EventKind::rejected(order_id, RejectReason::DuplicateOrderId),
            );
            return;
        }

        self.emit(events, EventKind::Accepted { order_id });
        let order_seq = self.next_seq;

        let mut remaining_qty = qty;
        self.match_order(order_id, side, price, &mut remaining_qty, events);

        if remaining_qty == 0 {
            self.emit(events, EventKind::Filled { order_id });
        } else {
            self.orders.insert(
                order_id,
                Order {
                    id: order_id,
                    side,
                    price,
                    qty,
                    remaining_qty,
                    sequence: order_seq,
                },
            );

            let same_side = match side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            same_side
                .entry(price)
                .or_default()
                .push(order_id, remaining_qty);
        }
    }

    /// Returns true if validation failed (invalid qty) and events were emitted.
    fn validate_order_qty(&mut self, id: OrderId, qty: Qty, events: &mut impl EventSink) -> bool {
        if qty == 0 {
            self.emit(
                events,
                EventKind::rejected(id, RejectReason::InvalidQuantity),
            );
            return true;
        }
        false
    }

    pub fn add_market_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        qty: Qty,
        events: &mut impl EventSink,
    ) {
        if self.validate_order_qty(order_id, qty, events) {
            return;
        }
        if self.orders.contains_key(&order_id) {
            self.emit(
                events,
                EventKind::rejected(order_id, RejectReason::DuplicateOrderId),
            );
            return;
        }

        self.emit(events, EventKind::Accepted { order_id });

        let price = match side {
            Side::Buy => Price::MAX,
            Side::Sell => Price::MIN,
        };
        let mut qty = qty;
        self.match_order(order_id, side, price, &mut qty, events);

        if qty == 0 {
            self.emit(events, EventKind::Filled { order_id });
        } else {
            self.emit(events, EventKind::cancelled(order_id, qty));
        }
    }

    pub fn cancel_order(&mut self, id: OrderId, events: &mut impl EventSink) {
        match self.orders.remove(&id) {
            None => {
                self.emit(events, EventKind::rejected(id, RejectReason::UnknownOrder));
            }
            Some(order) => {
                let book_side = match order.side {
                    Side::Buy => &mut self.bids,
                    Side::Sell => &mut self.asks,
                };

                if let Some(level) = book_side.get_mut(&order.price) {
                    level.remove(id, order.remaining_qty);

                    if level.is_empty() {
                        book_side.remove(&order.price);
                    }
                };

                self.emit(events, EventKind::cancelled(id, order.remaining_qty));
            }
        }
    }

    pub fn reduce_order(&mut self, id: OrderId, qty: Qty, events: &mut impl EventSink) {
        if self.validate_order_qty(id, qty, events) {
            return;
        }

        let Some(order) = self.orders.get_mut(&id) else {
            self.emit(events, EventKind::rejected(id, RejectReason::UnknownOrder));
            return;
        };

        if qty >= order.remaining_qty {
            self.cancel_order(id, events);
            return;
        }

        let side = order.side;
        let price = order.price;
        order.remaining_qty -= qty;

        let book_side = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        if let Some(level) = book_side.get_mut(&price) {
            level.reduce_qty(qty);
        }
    }

    fn match_order(
        &mut self,
        aggressor_order_id: OrderId,
        side: Side,
        price: Price,
        qty: &mut Qty,
        events: &mut impl EventSink,
    ) {
        while *qty > 0 {
            let (matched_price, price_level) = match side {
                Side::Buy => {
                    if let Some((ask_price, price_level)) = self.asks.iter_mut().next()
                        && *ask_price <= price
                    {
                        (*ask_price, price_level)
                    } else {
                        return;
                    }
                }
                Side::Sell => {
                    if let Some((bid_price, price_level)) = self.bids.iter_mut().next_back()
                        && price <= *bid_price
                    {
                        (*bid_price, price_level)
                    } else {
                        return;
                    }
                }
            };

            Self::fulfill_in_price_level(
                price_level,
                &mut self.orders,
                events,
                &mut self.next_seq,
                aggressor_order_id,
                qty,
            );

            if price_level.is_empty() {
                match side {
                    Side::Buy => {
                        self.asks.remove(&matched_price);
                    }
                    Side::Sell => {
                        self.bids.remove(&matched_price);
                    }
                }
            }
        }
    }

    pub(super) fn fulfill_in_price_level(
        price_level: &mut PriceLevel,
        orders: &mut HashMap<OrderId, Order>,
        events: &mut impl EventSink,
        next_seq: &mut u64,
        aggressor_order_id: OrderId,
        qty: &mut Qty,
    ) {
        while *qty > 0
            && let Some(passive_order_id) = price_level.front()
        {
            let passive_order = orders
                .get_mut(&passive_order_id)
                .expect("order exists in orders map");
            let passive_remaining_qty = passive_order.remaining_qty;
            let price = passive_order.price;

            let fill_qty = passive_remaining_qty.min(*qty);
            emit(
                events,
                next_seq,
                EventKind::Fill {
                    passive_order_id,
                    aggressor_order_id,
                    price,
                    qty: fill_qty,
                },
            );

            if *qty >= passive_remaining_qty {
                price_level.remove(passive_order_id, passive_remaining_qty);

                emit(
                    events,
                    next_seq,
                    EventKind::Filled {
                        order_id: passive_order_id,
                    },
                );
                orders.remove(&passive_order_id);

                *qty -= passive_remaining_qty;
            } else {
                price_level.reduce_qty(*qty);
                passive_order.remaining_qty -= *qty;
                *qty = 0;
            }
        }
    }
}

impl Default for LimitOrderBookV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl LimitOrderBook for LimitOrderBookV0 {
    fn with_config(_: (Price, Price), _: u64) -> Self {
        Self::new()
    }

    fn best_bid(&self) -> Option<(Price, Qty)> {
        LimitOrderBookV0::best_bid(self)
    }

    fn best_ask(&self) -> Option<(Price, Qty)> {
        LimitOrderBookV0::best_ask(self)
    }

    fn order(&self, id: OrderId) -> Option<&Order> {
        LimitOrderBookV0::order(self, id)
    }

    fn order_count(&self) -> u64 {
        LimitOrderBookV0::order_count(self)
    }

    fn depth(&mut self, side: Side, levels: u64) -> Vec<(Price, Qty)> {
        LimitOrderBookV0::depth(self, side, levels)
    }

    fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
        events: &mut impl EventSink,
    ) {
        LimitOrderBookV0::add_limit_order(self, order_id, side, price, qty, events)
    }

    fn add_market_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        qty: Qty,
        events: &mut impl EventSink,
    ) {
        LimitOrderBookV0::add_market_order(self, order_id, side, qty, events)
    }

    fn cancel_order(&mut self, order_id: OrderId, events: &mut impl EventSink) {
        LimitOrderBookV0::cancel_order(self, order_id, events)
    }

    fn reduce_order(&mut self, order_id: OrderId, qty: Qty, events: &mut impl EventSink) {
        LimitOrderBookV0::reduce_order(self, order_id, qty, events)
    }
}
