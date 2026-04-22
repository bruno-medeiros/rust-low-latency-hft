use crate::types::{OrderId, Price, Qty};
use serde::{Deserialize, Serialize};

/// Sink for order book events. Implementations receive events from [`crate::LimitOrderBook`] commands.
pub trait BookEventSink {
    fn push(&mut self, event: BookEvent);
}

impl BookEventSink for Vec<BookEvent> {
    fn push(&mut self, event: BookEvent) {
        Vec::push(self, event);
    }
}

/// Event sink that counts events per [`BookEventKind`] variant. Useful for benchmarks or when only
/// aggregate counts are needed.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountingEventSink {
    pub accepted: u64,
    pub rejected: u64,
    pub fill: u64,
    pub filled: u64,
    pub cancelled: u64,
}

impl BookEventSink for CountingEventSink {
    fn push(&mut self, event: BookEvent) {
        match event.kind {
            BookEventKind::Accepted { .. } => self.accepted += 1,
            BookEventKind::Rejected { .. } => self.rejected += 1,
            BookEventKind::Fill { .. } => self.fill += 1,
            BookEventKind::Filled { .. } => self.filled += 1,
            BookEventKind::Cancelled { .. } => self.cancelled += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookEvent {
    pub sequence: u64,
    pub kind: BookEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookEventKind {
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

impl BookEventKind {
    pub fn rejected(order_id: OrderId, reason: RejectReason) -> Self {
        BookEventKind::Rejected { order_id, reason }
    }

    pub fn cancelled(order_id: OrderId, remaining_qty: Qty) -> Self {
        BookEventKind::Cancelled {
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
