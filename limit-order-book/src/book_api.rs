use crate::event::{Event, EventSink};
use crate::order::Order;
use crate::types::{OrderId, Price, Qty, Side};

pub trait LimitOrderBook {
    /// Create a new book with implementation-specific pre-allocation settings.
    fn with_config(price_range: (Price, Price), order_capacity: u64) -> Self;

    /// Return the best bid as `(price, aggregate_qty)`.
    fn best_bid(&self) -> Option<(Price, Qty)>;

    /// Return the best ask as `(price, aggregate_qty)`.
    fn best_ask(&self) -> Option<(Price, Qty)>;

    fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    /// Return a resting order by ID, if present.
    fn order(&self, id: OrderId) -> Option<&Order>;

    /// Return the number of resting orders currently stored.
    fn order_count(&self) -> u64;

    /// Return up to `levels` price levels on one side of book as `(price, aggregate_qty)`.
    fn depth(&mut self, side: Side, levels: u64) -> Vec<(Price, Qty)>;

    /// Insert a limit order with side, price, quantity, and ID.
    ///
    /// If prices cross, the incoming order matches immediately against opposite-side liquidity.
    /// Any remaining quantity rests in the book.
    ///
    /// Implementations emit `Accepted`, `Fill`, `Filled`, `Cancelled`, and/or `Rejected`
    /// events into `events` depending on outcome.
    fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
        events: &mut impl EventSink,
    );

    /// Insert a market order with side, quantity, and ID.
    ///
    /// Market orders match against the opposite side at best available prices until filled.
    /// Any unfilled remainder is cancelled and never rests.
    ///
    /// Implementations emit `Accepted`, `Fill`, `Filled`, `Cancelled`, and/or `Rejected`
    /// events into `events` depending on outcome.
    fn add_market_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        qty: Qty,
        events: &mut impl EventSink,
    );

    /// Remove a resting order by ID.
    ///
    /// Rejects with `UnknownOrder` if the order ID is not currently resting.
    fn cancel_order(&mut self, order_id: OrderId, events: &mut impl EventSink);

    /// Reduce an existing resting order by `qty`.
    /// If `qty` is greater than or equal to the remaining quantity, this removes the order.
    fn reduce_order(&mut self, order_id: OrderId, qty: Qty, events: &mut impl EventSink);

    fn add_limit_order_vec(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        self.add_limit_order(order_id, side, price, qty, &mut events);
        events
    }

    fn add_market_order_vec(&mut self, order_id: OrderId, side: Side, qty: Qty) -> Vec<Event> {
        let mut events = Vec::new();
        self.add_market_order(order_id, side, qty, &mut events);
        events
    }

    fn cancel_order_vec(&mut self, order_id: OrderId) -> Vec<Event> {
        let mut events = Vec::new();
        self.cancel_order(order_id, &mut events);
        events
    }

    fn reduce_order_vec(&mut self, order_id: OrderId, qty: Qty) -> Vec<Event> {
        let mut events = Vec::new();
        self.reduce_order(order_id, qty, &mut events);
        events
    }
}
