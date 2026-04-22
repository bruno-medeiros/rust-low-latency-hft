use crate::LimitOrderBook;
use crate::book_v1::orders::BookOrders;
use crate::book_v1::price_level::PriceLevel;
use crate::event::{BookEvent, BookEventKind, BookEventSink, RejectReason};
use crate::order::Order;
use crate::types::{OrderId, OrderSlabKey, Price, Qty, Side};

//noinspection DuplicatedCode
fn emit(events: &mut impl BookEventSink, seq: &mut u64, kind: BookEventKind) {
    let s = *seq;
    *seq += 1;
    events.push(BookEvent { sequence: s, kind })
}

#[derive(Debug)]
pub struct OrderSlot {
    pub next: Option<OrderSlabKey>,
    pub prev: Option<OrderSlabKey>,
    pub order: Order,
}

/// An efficient PriceLevels data structure that preallocates
/// a vec for a range of prices (ticks)
// TODO: back with mmap(MAP_HUGETLB) on Linux to reduce TLB misses on the dense price array
#[derive(Debug)]
pub struct PriceLevels {
    /// Levels array (fixed-size)
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
        let level_capacity = (max_price - min_price + 1) as usize;

        Self {
            levels: (0..level_capacity)
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

    pub fn bids_iter(&mut self) -> impl Iterator<Item = &PriceLevel> {
        let end = self.best_bid_ix.map(|i| i + 1).unwrap_or(0);
        self.levels[0..end].iter().rev().filter_map(Option::as_ref)
    }

    pub fn bids_iter_mut(&mut self) -> impl Iterator<Item = &mut PriceLevel> {
        let end = self.best_bid_ix.map(|i| i + 1).unwrap_or(0);
        self.levels[0..end]
            .iter_mut()
            .rev()
            .filter_map(Option::as_mut)
    }

    pub fn asks_iter(&mut self) -> impl Iterator<Item = &PriceLevel> {
        let start = self.best_ask_ix.unwrap_or(self.levels.len());
        self.levels[start..].iter().filter_map(Option::as_ref)
    }

    pub fn asks_iter_mut(&mut self) -> impl Iterator<Item = &mut PriceLevel> {
        let start = self.best_ask_ix.unwrap_or(self.levels.len());
        self.levels[start..].iter_mut().filter_map(Option::as_mut)
    }

    fn best_bid_price(&self) -> Option<Price> {
        self.best_bid_ix.map(|ix| self.base_price + ix as Price)
    }

    fn best_ask_price(&self) -> Option<Price> {
        self.best_ask_ix.map(|ix| self.base_price + ix as Price)
    }

    pub fn existing_level(&mut self, price: Price) -> &mut PriceLevel {
        let levels_ix = (price - self.base_price) as usize;
        self.levels[levels_ix].as_mut().expect("price level exists")
    }

    pub fn add_order(
        &mut self,
        key: OrderSlabKey,
        side: Side,
        price: Price,
        qty: Qty,
    ) -> Option<OrderSlabKey> {
        let ix = (price - self.base_price) as usize;

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
        let level = self.levels[ix].get_or_insert_with(|| PriceLevel::new(price));
        level.append_order(key, qty)
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
                        .bids_iter_mut()
                        .next()
                        .map(|pl| (pl.price - base_price) as usize);
                    self.best_bid_ix = new_best_bid;
                }
                Side::Sell => {
                    let new_ask_bid = self
                        .asks_iter_mut()
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
    pub price_levels: PriceLevels,
    pub orders: BookOrders,
    pub next_seq: u64,
}

//noinspection DuplicatedCode
impl LimitOrderBookV1 {
    /// Create a new Limit Order Book Book with given price_range
    /// and initial order capacity (pre-allocation hint).
    pub fn new(price_range: (Price, Price), order_capacity: usize) -> Self {
        Self {
            price_levels: PriceLevels::new(price_range),
            orders: BookOrders::new(order_capacity),
            next_seq: 0,
        }
    }

    fn emit(&mut self, events: &mut impl BookEventSink, kind: BookEventKind) {
        emit(events, &mut self.next_seq, kind)
    }

    // -- queries -----------------------------------------------------------

    pub fn best_bid(&self) -> Option<(Price, Qty)> {
        self.price_levels
            .best_bid()
            .map(|pl| (pl.price, pl.total_qty))
    }

    pub fn best_ask(&self) -> Option<(Price, Qty)> {
        self.price_levels
            .best_ask()
            .map(|pl| (pl.price, pl.total_qty))
    }

    pub fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    pub fn order(&self, id: OrderId) -> Option<&Order> {
        self.orders.order(id)
    }

    pub fn order_count(&self) -> u64 {
        self.orders.order_count()
    }

    pub fn depth(&mut self, side: Side, levels: u64) -> Vec<(Price, Qty)> {
        match side {
            Side::Buy => (self.price_levels.bids_iter_mut())
                .take(levels as usize)
                .map(|pl| (pl.price, pl.total_qty))
                .collect(),
            Side::Sell => (self.price_levels.asks_iter_mut())
                .take(levels as usize)
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
        events: &mut impl BookEventSink,
    ) {
        if self.validate_order_qty(order_id, qty, events) {
            return;
        }
        if price == 0 {
            self.emit(
                events,
                BookEventKind::rejected(order_id, RejectReason::InvalidPrice),
            );
            return;
        }

        if self.order(order_id).is_some() {
            self.emit(
                events,
                BookEventKind::rejected(order_id, RejectReason::DuplicateOrderId),
            );
            return;
        }

        self.emit(events, BookEventKind::Accepted { order_id });
        let order_seq = self.next_seq;

        let mut remaining_qty = qty;
        self.match_order(order_id, side, price, &mut remaining_qty, events);

        if remaining_qty == 0 {
            self.emit(events, BookEventKind::Filled { order_id });
        } else {
            let order = Order {
                id: order_id,
                side,
                price,
                qty,
                remaining_qty,
                sequence: order_seq,
            };

            let key = self.orders.add_order(order);
            let old_tail_key = self.price_levels.add_order(key, side, price, qty);
            if let Some(old_tail_key) = old_tail_key {
                self.orders.slot_mut(old_tail_key).next = Some(key);
            }
            self.orders.slot_mut(key).prev = old_tail_key;
        }
    }

    /// Returns true if validation failed (invalid qty) and events were emitted.
    fn validate_order_qty(&mut self, id: OrderId, qty: Qty, events: &mut impl BookEventSink) -> bool {
        if qty == 0 {
            self.emit(
                events,
                BookEventKind::rejected(id, RejectReason::InvalidQuantity),
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
        events: &mut impl BookEventSink,
    ) {
        if self.validate_order_qty(order_id, qty, events) {
            return;
        }
        if self.order(order_id).is_some() {
            self.emit(
                events,
                BookEventKind::rejected(order_id, RejectReason::DuplicateOrderId),
            );
            return;
        }

        self.emit(events, BookEventKind::Accepted { order_id });

        let price = match side {
            Side::Buy => Price::MAX,
            Side::Sell => Price::MIN,
        };
        let mut qty = qty;
        self.match_order(order_id, side, price, &mut qty, events);

        if qty == 0 {
            self.emit(events, BookEventKind::Filled { order_id });
        } else {
            self.emit(events, BookEventKind::cancelled(order_id, qty));
        }
    }

    pub fn cancel_order(&mut self, order_id: OrderId, events: &mut impl BookEventSink) {
        match self.orders.order_key(order_id) {
            None => {
                self.emit(
                    events,
                    BookEventKind::rejected(order_id, RejectReason::UnknownOrder),
                );
            }
            Some(order_key) => {
                let order_slot = self.remove_order(order_key);
                self.emit(
                    events,
                    BookEventKind::cancelled(order_id, order_slot.order.remaining_qty),
                );
            }
        }
    }

    pub fn reduce_order(&mut self, order_id: OrderId, qty: Qty, events: &mut impl BookEventSink) {
        if self.validate_order_qty(order_id, qty, events) {
            return;
        }

        let Some(order_key) = self.orders.order_key(order_id) else {
            self.emit(
                events,
                BookEventKind::rejected(order_id, RejectReason::UnknownOrder),
            );
            return;
        };

        let remaining_qty = self.orders.slot(order_key).order.remaining_qty;
        if qty >= remaining_qty {
            let order_slot = self.remove_order(order_key);
            self.emit(
                events,
                BookEventKind::cancelled(order_id, order_slot.order.remaining_qty),
            );
            return;
        }

        let order = &mut self.orders.slot_mut(order_key).order;
        let price = order.price;
        order.remaining_qty -= qty;
        self.price_levels.existing_level(price).reduce_qty(qty);
    }

    fn remove_order(&mut self, order_key: OrderSlabKey) -> OrderSlot {
        let order_slot = self.orders.remove_by_key(order_key);

        let price_level = self.price_levels.existing_level(order_slot.order.price);
        price_level.remove(order_key, &order_slot);
        let order = &order_slot.order;
        self.price_levels.remove_if_empty(order.price, order.side);

        order_slot
    }

    #[allow(clippy::too_many_arguments)]
    fn match_order(
        &mut self,
        aggressor_order_id: OrderId,
        side: Side,
        price: Price,
        qty: &mut Qty,
        events: &mut impl BookEventSink,
    ) {
        while *qty > 0 {
            let price_level = match side {
                Side::Buy => {
                    if let Some(price_level) = self.price_levels.asks_iter_mut().next()
                        && price_level.price <= price
                    {
                        price_level
                    } else {
                        return;
                    }
                }
                Side::Sell => {
                    if let Some(price_level) = self.price_levels.bids_iter_mut().next()
                        && price <= price_level.price
                    {
                        price_level
                    } else {
                        return;
                    }
                }
            };
            let price_level = &*price_level;
            let matched_price = price_level.price;

            self.match_order_at_matched_price(aggressor_order_id, side, matched_price, qty, events);
        }
    }

    pub fn match_order_at_matched_price(
        &mut self,
        aggressor_order_id: OrderId,
        side: Side,
        matched_price: Price,
        qty: &mut Qty,
        events: &mut impl BookEventSink,
    ) {
        let price_level = self.price_levels.existing_level(matched_price);

        while *qty > 0
            && let Some(passive_key) = price_level.front()
        {
            let passive_order_id = self.orders.slot(passive_key).order.id;
            let passive_remaining_qty = self.orders.slot(passive_key).order.remaining_qty;

            let fill_qty = passive_remaining_qty.min(*qty);
            emit(
                events,
                &mut self.next_seq,
                BookEventKind::Fill {
                    passive_order_id,
                    aggressor_order_id,
                    price: matched_price,
                    qty: fill_qty,
                },
            );

            if *qty >= passive_remaining_qty {
                let passive_slot = self.orders.slot(passive_key);
                price_level.remove(passive_key, passive_slot);

                self.orders.remove_by_key(passive_key);

                emit(
                    events,
                    &mut self.next_seq,
                    BookEventKind::Filled {
                        order_id: passive_order_id,
                    },
                );

                *qty -= passive_remaining_qty;
            } else {
                price_level.reduce_qty(*qty);
                self.orders.slot_mut(passive_key).order.remaining_qty -= *qty;
                *qty = 0;
            }
        }

        let passive_side = side.opposite();
        self.price_levels
            .remove_if_empty(matched_price, passive_side);
    }
}

impl LimitOrderBook for LimitOrderBookV1 {
    fn with_config(price_range: (Price, Price), order_capacity: u64) -> Self {
        Self::new(price_range, order_capacity as usize)
    }

    fn best_bid(&self) -> Option<(Price, Qty)> {
        LimitOrderBookV1::best_bid(self)
    }

    fn best_ask(&self) -> Option<(Price, Qty)> {
        LimitOrderBookV1::best_ask(self)
    }

    fn order(&self, id: OrderId) -> Option<&Order> {
        LimitOrderBookV1::order(self, id)
    }

    fn order_count(&self) -> u64 {
        LimitOrderBookV1::order_count(self)
    }

    fn depth(&mut self, side: Side, levels: u64) -> Vec<(Price, Qty)> {
        LimitOrderBookV1::depth(self, side, levels)
    }

    fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
        events: &mut impl BookEventSink,
    ) {
        LimitOrderBookV1::add_limit_order(self, order_id, side, price, qty, events)
    }

    fn add_market_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        qty: Qty,
        events: &mut impl BookEventSink,
    ) {
        LimitOrderBookV1::add_market_order(self, order_id, side, qty, events)
    }

    fn cancel_order(&mut self, order_id: OrderId, events: &mut impl BookEventSink) {
        LimitOrderBookV1::cancel_order(self, order_id, events)
    }

    fn reduce_order(&mut self, order_id: OrderId, qty: Qty, events: &mut impl BookEventSink) {
        LimitOrderBookV1::reduce_order(self, order_id, qty, events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl LimitOrderBookV1 {
        fn lvl_head_tail(&self, i: usize) -> (Option<OrderId>, Option<OrderId>) {
            let x = self.price_levels.levels[i].as_ref().unwrap();
            let head_id = x.order_head.map(|k| self.orders.slot(k).order.id);
            let tail_id = x.order_tail.map(|k| self.orders.slot(k).order.id);
            (head_id, tail_id)
        }

        fn lvl_orders(&self, i: usize) -> Vec<OrderId> {
            let lvl = self.price_levels.levels[i].as_ref().unwrap();

            let tail = lvl.order_tail;
            let mut next = lvl.order_head;
            let mut prev = None;

            let mut orders = vec![];
            while let Some(key) = next {
                let slot = self.orders.slot(key);
                orders.push(slot.order.id);
                assert!(slot.prev == prev);
                prev = Some(key);
                next = slot.next;
            }
            assert_eq!(tail, prev);
            orders
        }
    }

    #[test]
    fn order_chaining_linked_list() {
        let mut book = LimitOrderBookV1::new((0, 10_000), 1_000);
        book.add_limit_order_vec(1, Side::Buy, 100, 10);
        assert!(book.lvl_orders(100) == vec![1]);

        book.add_limit_order_vec(2, Side::Buy, 100, 5);
        assert!(book.lvl_orders(100) == vec![1, 2]);

        book.add_limit_order_vec(3, Side::Buy, 100, 5);
        assert!(book.lvl_orders(100) == vec![1, 2, 3]);

        // --- Test cancelling:

        // Cancel middle
        book.cancel_order_vec(2);
        assert_eq!(book.lvl_head_tail(100), (Some(1), Some(3)));
        assert!(book.lvl_orders(100) == vec![1, 3]);

        // Cancel head
        book.cancel_order_vec(1);
        assert!(book.lvl_orders(100) == vec![3]);

        // Cancel head-tail
        book.cancel_order_vec(3);
        assert!(book.price_levels.levels[100].is_none());
    }

    #[test]
    fn order_chaining_linked_list_after_matching() {
        let mut book = LimitOrderBookV1::new((0, 10_000), 1_000);
        book.add_limit_order_vec(1, Side::Buy, 100, 10);
        book.add_limit_order_vec(2, Side::Buy, 100, 5);
        book.add_limit_order_vec(3, Side::Buy, 100, 5);
        assert!(book.lvl_orders(100) == vec![1, 2, 3]);

        // Matching follows a different path than cancelling orders

        // Fill order 1
        book.add_limit_order_vec(11, Side::Sell, 100, 10);
        assert!(book.lvl_orders(100) == vec![2, 3]);

        // Fill order 2 (partial)
        book.add_limit_order_vec(12, Side::Sell, 100, 2);
        assert!(book.lvl_orders(100) == vec![2, 3]);

        // Fill order 2 (partial)
        book.add_limit_order_vec(13, Side::Sell, 100, 3);
        assert!(book.lvl_orders(100) == vec![3]);

        // Fill order 3
        book.add_limit_order_vec(14, Side::Sell, 100, 5);
        assert!(book.price_levels.levels[100].is_none());
    }
}
