use limit_order_book::types::{OrderId, Price, Qty, Side};

/// Command dispatched to the matching engine via the pipeline.
///
/// These are order-entry messages: instructions that cause the matching engine
/// to act (insert + match, or cancel). Intentionally `Copy` and compact so they
/// can be pushed through the SPSC queue without allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderCommand {
    /// Submit a new limit order. The matching engine will attempt to match
    /// the aggressor side immediately; any unmatched remainder rests on the book.
    NewOrder {
        order_id: OrderId,
        side: Side,
        price: Price,
        qty: Qty,
    },
    /// Submit a market order. Matches immediately against resting orders;
    /// any unmatched remainder is rejected (no resting).
    MarketOrder {
        order_id: OrderId,
        side: Side,
        qty: Qty,
    },
    /// Cancel a resting order by ID.
    CancelOrder { order_id: OrderId },
}
