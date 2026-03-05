use crate::types::{OrderId, Price, Qty, Side};

#[derive(Debug, Clone)]
pub struct Order {
    /// Unique identifier assigned by the caller; used to reference
    /// this order in cancel/modify requests and fill reports.
    pub id: OrderId,
    pub side: Side,
    pub price: Price,
    pub qty: Qty,
    pub remaining_qty: Qty,
    /// Monotonically increasing value assigned on insertion;
    /// determines time priority among orders at the same price level.
    pub sequence: u64,
}
