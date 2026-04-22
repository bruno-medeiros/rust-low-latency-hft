use crate::message::{ItchMessage, Side as FeedSide};
use limit_order_book::event::EventSink;
use limit_order_book::types::{OrderId, Side as BookSide};
use limit_order_book::LimitOrderBook;
use thiserror::Error;

/// Outcome of attempting to apply one feed message to a limit-order-book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedBookAction {
    AppliedAdd,
    AppliedCancel,
    AppliedReduce,
    IgnoredSystemEvent,
}

/// Explicitly unsupported or invalid feed-to-book mapping cases.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FeedBookError {
    #[error("unsupported feed message kind for direct book application")]
    UnsupportedMessageKind,
    #[error("cancel for unknown order id {0}")]
    UnknownOrder(OrderId),
    #[error("invalid reduction quantity 0 for order {0}")]
    InvalidReductionQty(OrderId),
}

/// Adapter that converts feed events into existing book commands.
#[derive(Debug, Default)]
pub struct FeedBookAdapter;

impl FeedBookAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn apply<B: LimitOrderBook>(
        &mut self,
        book: &mut B,
        msg: &ItchMessage<'_>,
        events: &mut impl EventSink,
    ) -> Result<FeedBookAction, FeedBookError> {
        match msg {
            ItchMessage::SystemEvent { .. } => Ok(FeedBookAction::IgnoredSystemEvent),
            ItchMessage::AddOrder {
                oid,
                side,
                qty,
                price,
                symbol: _,
            } => {
                book.add_limit_order(
                    *oid,
                    map_side(*side),
                    u64::from(*price),
                    u64::from(*qty),
                    events,
                );
                Ok(FeedBookAction::AppliedAdd)
            }
            ItchMessage::OrderCanceled { oid, qty } => {
                let Some(existing) = book.order(*oid) else {
                    return Err(FeedBookError::UnknownOrder(*oid));
                };
                let cancel_qty = u64::from(*qty);
                if cancel_qty == 0 {
                    return Err(FeedBookError::InvalidReductionQty(*oid));
                }
                if existing.remaining_qty == cancel_qty {
                    book.cancel_order(*oid, events);
                    Ok(FeedBookAction::AppliedCancel)
                } else {
                    book.reduce_order(*oid, cancel_qty, events);
                    Ok(FeedBookAction::AppliedReduce)
                }
            }
            ItchMessage::OrderExecuted { oid, qty } => {
                let Some(_existing) = book.order(*oid) else {
                    return Err(FeedBookError::UnknownOrder(*oid));
                };
                let exec_qty = u64::from(*qty);
                if exec_qty == 0 {
                    return Err(FeedBookError::InvalidReductionQty(*oid));
                }
                book.reduce_order(*oid, exec_qty, events);
                Ok(FeedBookAction::AppliedReduce)
            }
            ItchMessage::Trade { .. } => Err(FeedBookError::UnsupportedMessageKind),
        }
    }
}

fn map_side(side: FeedSide) -> BookSide {
    match side {
        FeedSide::Buy => BookSide::Buy,
        FeedSide::Sell => BookSide::Sell,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{ItchMessage, Side as FeedSide};
    use limit_order_book::CountingEventSink;
    use limit_order_book::LimitOrderBookV1;

    #[test]
    fn applies_add_and_full_cancel_messages_to_book() {
        let mut book = LimitOrderBookV1::new((90, 110), 32);
        let mut adapter = FeedBookAdapter::new();
        let mut events = CountingEventSink::default();

        let add = ItchMessage::AddOrder {
            oid: 10,
            side: FeedSide::Buy,
            qty: 7,
            price: 100,
            symbol: "AAPL",
        };

        let action = adapter.apply(&mut book, &add, &mut events).unwrap();
        assert_eq!(action, FeedBookAction::AppliedAdd);
        assert_eq!(book.order_count(), 1);
        assert_eq!(book.best_bid(), Some((100, 7)));

        let cancel = ItchMessage::OrderCanceled { oid: 10, qty: 7 };
        let action = adapter.apply(&mut book, &cancel, &mut events).unwrap();
        assert_eq!(action, FeedBookAction::AppliedCancel);
        assert_eq!(book.order_count(), 0);
        assert_eq!(book.best_bid(), None);

        assert_eq!(events.accepted, 1);
        assert_eq!(events.cancelled, 1);
    }

    #[test]
    fn applies_partial_cancel_and_order_executed_via_reduce_order() {
        let mut book = LimitOrderBookV1::new((90, 110), 32);
        let mut adapter = FeedBookAdapter::new();
        let mut events = CountingEventSink::default();

        let add = ItchMessage::AddOrder {
            oid: 20,
            side: FeedSide::Sell,
            qty: 10,
            price: 101,
            symbol: "AAPL",
        };
        adapter.apply(&mut book, &add, &mut events).unwrap();
        assert_eq!(book.best_ask(), Some((101, 10)));

        let partial_cancel = ItchMessage::OrderCanceled { oid: 20, qty: 3 };
        adapter
            .apply(&mut book, &partial_cancel, &mut events)
            .unwrap();
        assert_eq!(book.best_ask(), Some((101, 7)));

        let executed = ItchMessage::OrderExecuted { oid: 20, qty: 2 };
        adapter.apply(&mut book, &executed, &mut events).unwrap();
        assert_eq!(book.best_ask(), Some((101, 5)));
    }
}
