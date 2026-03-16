use crate::types::{OrderId, Price, Qty};
use serde::{Deserialize, Serialize};

/// Sink for order book events. Implementations receive events from [`crate::LimitOrderBook`] commands.
pub trait EventSink {
    fn push(&mut self, event: Event);
}

impl EventSink for Vec<Event> {
    fn push(&mut self, event: Event) {
        Vec::push(self, event);
    }
}

/// Event sink that counts events per [`EventKind`] variant. Useful for benchmarks or when only
/// aggregate counts are needed.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountingEventSink {
    pub accepted: u64,
    pub rejected: u64,
    pub fill: u64,
    pub filled: u64,
    pub cancelled: u64,
}

impl EventSink for CountingEventSink {
    fn push(&mut self, event: Event) {
        match event.kind {
            EventKind::Accepted { .. } => self.accepted += 1,
            EventKind::Rejected { .. } => self.rejected += 1,
            EventKind::Fill { .. } => self.fill += 1,
            EventKind::Filled { .. } => self.filled += 1,
            EventKind::Cancelled { .. } => self.cancelled += 1,
        }
    }
}

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

impl EventKind {
    pub fn rejected(order_id: OrderId, reason: RejectReason) -> Self {
        EventKind::Rejected { order_id, reason }
    }

    pub fn cancelled(order_id: OrderId, remaining_qty: Qty) -> Self {
        EventKind::Cancelled {
            order_id,
            remaining_qty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    DuplicateOrderId,
    InvalidPrice,
    InvalidQuantity,
    UnknownOrder,
}
