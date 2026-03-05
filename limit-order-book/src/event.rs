use crate::types::{OrderId, Price, Qty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub kind: EventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Accepted {
        order_id: OrderId,
    },
    Rejected {
        order_id: OrderId,
        reason: RejectReason,
    },
    Fill {
        aggressor_order_id: OrderId,
        passive_order_id: OrderId,
        price: Price,
        qty: Qty,
    },
    Filled {
        order_id: OrderId,
    },
    Cancelled {
        order_id: OrderId,
        remaining_qty: Qty,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    DuplicateOrderId,
    InvalidPrice,
    InvalidQuantity,
    UnknownOrder,
}
