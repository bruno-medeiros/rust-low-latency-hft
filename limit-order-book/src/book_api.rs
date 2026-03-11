use crate::event::Event;
use crate::order::Order;
use crate::types::{OrderId, Price, Qty, Side};

pub trait LimitOrderBook {
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
    ) -> Vec<Event>;

    fn add_market_order(&mut self, order_id: OrderId, side: Side, qty: Qty) -> Vec<Event>;

    fn cancel_order(&mut self, order_id: OrderId) -> Vec<Event>;
}
