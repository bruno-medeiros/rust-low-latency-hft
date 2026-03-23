use crate::event::{Event, EventSink};
use crate::order::Order;
use crate::types::{OrderId, Price, Qty, Side};

pub trait LimitOrderBook {
    fn with_config(price_range: (Price, Price), order_capacity: u64) -> Self;

    fn best_bid(&self) -> Option<(Price, Qty)>;

    fn best_ask(&self) -> Option<(Price, Qty)>;

    fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask.saturating_sub(bid)),
            _ => None,
        }
    }

    fn order(&self, id: OrderId) -> Option<&Order>;

    fn order_count(&self) -> u64;

    fn depth(&mut self, side: Side, levels: u64) -> Vec<(Price, Qty)>;

    fn add_limit_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
        events: &mut impl EventSink,
    );

    fn add_market_order(
        &mut self,
        order_id: OrderId,
        side: Side,
        qty: Qty,
        events: &mut impl EventSink,
    );

    fn cancel_order(&mut self, order_id: OrderId, events: &mut impl EventSink);

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
}
