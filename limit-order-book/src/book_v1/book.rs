use crate::LimitOrderBook;
use crate::book_v1::price_level::PriceLevel;
use crate::event::{Event, EventKind, RejectReason};
use crate::order::Order;
use crate::types::{OrderId, Price, Qty, Side};
use std::collections::HashMap;

fn emit(seq: &mut u64, kind: EventKind) -> Event {
    let s = *seq;
    *seq += 1;
    Event { sequence: s, kind }
}

/// An efficient PriceLevels data structure that preallocates
/// a vec for a range of prices (ticks)
#[derive(Debug)]
pub struct PriceLevels {
    pub levels: Box<[Option<PriceLevel>]>,
    /// lowest possible price
    pub base_price: Price,
    /// Index of best bid.
    pub best_bid_ix: Option<usize>,
    /// Index of best ask. Must be > best_bid_ix
    pub best_ask_ix: Option<usize>,
}

impl PriceLevels {
    pub fn new(price_range: (Price, Price)) -> Self {
        let (min_price, max_price) = price_range;
        assert!(min_price <= max_price);
        let capacity = (max_price - min_price + 1) as usize;
        Self {
            levels: (0..capacity)
                .map(|_| None)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            base_price: min_price,
            best_ask_ix: None,
            best_bid_ix: None,
        }
    }

    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.best_bid_ix.map(|i| self.levels[i].as_ref().unwrap())
    }

    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.best_ask_ix.map(|i| self.levels[i].as_ref().unwrap())
    }

    pub fn bids_iter(&mut self) -> impl Iterator<Item = &mut PriceLevel> {
        let end = self.best_bid_ix.map(|i| i + 1).unwrap_or(0);
        self.levels[0..end]
            .iter_mut()
            .rev()
            .filter_map(Option::as_mut)
    }

    pub fn asks_iter(&mut self) -> impl Iterator<Item = &mut PriceLevel> {
        let start = self.best_ask_ix.unwrap_or(self.levels.len());
        self.levels[start..].iter_mut().filter_map(Option::as_mut)
    }

    fn best_bid_price(&self) -> Option<Price> {
        self.best_bid_ix.map(|ix| self.base_price + ix as Price)
    }

    fn best_ask_price(&self) -> Option<Price> {
        self.best_ask_ix.map(|ix| self.base_price + ix as Price)
    }

    pub fn add_order(&mut self, order_id: OrderId, side: Side, price: Price, qty: Qty) {
        let ix = (price - self.base_price) as usize;
        let level = self.levels[ix].get_or_insert_with(|| PriceLevel::new(price));
        level.push(order_id, qty);

        match side {
            Side::Buy => {
                if self
                    .best_bid_price()
                    .is_none_or(|best_bid| price > best_bid)
                {
                    self.best_bid_ix = Some(ix);
                }
            }
            Side::Sell => {
                if self
                    .best_ask_price()
                    .is_none_or(|best_ask| price < best_ask)
                {
                    self.best_ask_ix = Some(ix);
                }
            }
        }
    }

    pub fn remove_order(&mut self, order_id: OrderId, side: Side, price: Price, qty: Qty) {
        let ix = (price - self.base_price) as usize;

        match &mut self.levels[ix] {
            None => {}
            Some(price_level) => {
                price_level.remove(order_id, qty);
                self.remove_if_empty(price, side);
            }
        }
    }

    pub fn remove_if_empty(&mut self, price: Price, side: Side) {
        let ix = (price - self.base_price) as usize;

        if let Some(price_level) = &mut self.levels[ix]
            && price_level.is_empty()
        {
            self.levels[ix] = None;

            let base_price = self.base_price;

            // Update best_bid/best_ask when a price level is removed
            match side {
                Side::Buy => {
                    let new_best_bid = self
                        .bids_iter()
                        .next()
                        .map(|pl| (pl.price - base_price) as usize);
                    self.best_bid_ix = new_best_bid;
                }
                Side::Sell => {
                    let new_ask_bid = self
                        .asks_iter()
                        .next()
                        .map(|pl| (pl.price - base_price) as usize);
                    self.best_ask_ix = new_ask_bid;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct LimitOrderBookV1 {
    pub(crate) levels: PriceLevels,
    pub(crate) orders: HashMap<OrderId, Order>,
    pub(crate) next_seq: u64,
}

//noinspection DuplicatedCode
impl LimitOrderBookV1 {
    pub fn new(price_range: (Price, Price)) -> Self {
        Self {
            levels: PriceLevels::new(price_range),
            orders: HashMap::new(),
            next_seq: 0,
        }
    }

    fn emit(&mut self, kind: EventKind) -> Event {
        emit(&mut self.next_seq, kind)
    }

    // -- queries -----------------------------------------------------------

    pub fn best_bid(&self) -> Option<(Price, Qty)> {
        self.levels.best_bid().map(|pl| (pl.price, pl.total_qty))
    }

    pub fn best_ask(&self) -> Option<(Price, Qty)> {
        self.levels.best_ask().map(|pl| (pl.price, pl.total_qty))
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

    pub fn depth(&mut self, side: Side, levels: usize) -> Vec<(Price, Qty)> {
        match side {
            Side::Buy => (self.levels.bids_iter())
                .take(levels)
                .map(|pl| (pl.price, pl.total_qty))
                .collect(),
            Side::Sell => (self.levels.asks_iter())
                .take(levels)
                .map(|pl| (pl.price, pl.total_qty))
                .collect(),
        }
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
            return vec![self.emit(EventKind::Rejected {
                order_id,
                reason: RejectReason::InvalidPrice,
            })];
        }

        if self.orders.contains_key(&order_id) {
            return vec![self.emit(EventKind::Rejected {
                order_id,
                reason: RejectReason::DuplicateOrderId,
            })];
        }
        let mut events = Vec::new();

        events.push(self.emit(EventKind::Accepted { order_id }));
        let order_seq = self.next_seq;

        let mut remaining_qty = qty;
        self.match_order(order_id, side, price, &mut remaining_qty, &mut events);

        if remaining_qty == 0 {
            events.push(self.emit(EventKind::Filled { order_id }));
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

            self.levels.add_order(order_id, side, price, remaining_qty);
        }

        events
    }

    fn validate_order_qty(&mut self, id: OrderId, qty: Qty) -> Option<Vec<Event>> {
        if qty == 0 {
            return Some(vec![self.emit(EventKind::Rejected {
                order_id: id,
                reason: RejectReason::InvalidQuantity,
            })]);
        }

        None
    }

    pub fn add_market_order(&mut self, order_id: OrderId, side: Side, qty: Qty) -> Vec<Event> {
        if let Some(value) = self.validate_order_qty(order_id, qty) {
            return value;
        }
        if self.orders.contains_key(&order_id) {
            return vec![self.emit(EventKind::Rejected {
                order_id,
                reason: RejectReason::DuplicateOrderId,
            })];
        }
        let mut events = Vec::new();

        events.push(self.emit(EventKind::Accepted { order_id }));

        let price = match side {
            Side::Buy => Price::MAX,
            Side::Sell => Price::MIN,
        };
        let mut qty = qty;
        self.match_order(order_id, side, price, &mut qty, &mut events);

        if qty == 0 {
            events.push(self.emit(EventKind::Filled { order_id }));
        } else {
            events.push(self.emit(EventKind::Cancelled {
                order_id,
                remaining_qty: qty,
            }));
        }

        events
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Vec<Event> {
        match self.orders.remove(&order_id) {
            None => {
                vec![self.emit(EventKind::Rejected {
                    order_id,
                    reason: RejectReason::UnknownOrder,
                })]
            }
            Some(order) => {
                let mut events = Vec::new();

                self.levels
                    .remove_order(order_id, order.side, order.price, order.qty);
                // self.levels.remove_order(order_id, order.side, order.price, order.remaining_qty);

                events.push(self.emit(EventKind::Cancelled {
                    order_id,
                    remaining_qty: order.remaining_qty,
                }));

                events
            }
        }
    }

    fn match_order(
        &mut self,
        aggressor_order_id: OrderId,
        side: Side,
        price: Price,
        qty: &mut Qty,
        events: &mut Vec<Event>,
    ) {
        while *qty > 0 {
            let price_level = match side {
                Side::Buy => {
                    if let Some(price_level) = self.levels.asks_iter().next()
                        && price_level.price <= price
                    {
                        price_level
                    } else {
                        return;
                    }
                }
                Side::Sell => {
                    if let Some(price_level) = self.levels.bids_iter().next()
                        && price <= price_level.price
                    {
                        price_level
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

            let price = price_level.price;
            // Clear level on the passive side (opposite of aggressor)
            let passive_side = match side {
                Side::Buy => Side::Sell,
                Side::Sell => Side::Buy,
            };
            self.levels.remove_if_empty(price, passive_side);
        }
    }

    pub(super) fn fulfill_in_price_level(
        price_level: &mut PriceLevel,
        orders: &mut HashMap<OrderId, Order>,
        events: &mut Vec<Event>,
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
            events.push(emit(
                next_seq,
                EventKind::Fill {
                    passive_order_id,
                    aggressor_order_id,
                    price,
                    qty: fill_qty,
                },
            ));

            if *qty >= passive_remaining_qty {
                price_level.remove(passive_order_id, passive_remaining_qty);

                events.push(emit(
                    next_seq,
                    EventKind::Filled {
                        order_id: passive_order_id,
                    },
                ));
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

impl LimitOrderBook for LimitOrderBookV1 {
    fn best_bid(&self) -> Option<(Price, Qty)> {
        LimitOrderBookV1::best_bid(self)
    }

    fn best_ask(&self) -> Option<(Price, Qty)> {
        LimitOrderBookV1::best_ask(self)
    }

    fn order(&self, id: OrderId) -> Option<&Order> {
        LimitOrderBookV1::order(self, id)
    }

    fn order_count(&self) -> usize {
        LimitOrderBookV1::order_count(self)
    }

    fn depth(&mut self, side: Side, levels: usize) -> Vec<(Price, Qty)> {
        LimitOrderBookV1::depth(self, side, levels)
    }

    fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
    ) -> Vec<Event> {
        LimitOrderBookV1::add_limit_order(self, order_id, side, price, qty)
    }

    fn add_market_order(&mut self, order_id: OrderId, side: Side, qty: Qty) -> Vec<Event> {
        LimitOrderBookV1::add_market_order(self, order_id, side, qty)
    }

    fn cancel_order(&mut self, order_id: OrderId) -> Vec<Event> {
        LimitOrderBookV1::cancel_order(self, order_id)
    }
}
