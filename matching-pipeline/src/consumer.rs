use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use limit_order_book::event::CountingEventSink;
use limit_order_book::{BookEventSink, LimitOrderBook};
use lockfree_queue::spsc::SpscConsumer;

use crate::command::OrderCommand;
use crate::pipeline::PipelineResult;

/// Consumer thread: pops [`OrderCommand`]s from the SPSC queue, dispatches
/// them to the [`LimitOrderBook`] matching engine, and builds the
/// [`PipelineResult`].
pub(crate) fn consume<B: LimitOrderBook>(
    mut consumer: SpscConsumer<OrderCommand>,
    done: Arc<AtomicBool>,
    mut book: B,
) -> PipelineResult {
    let mut events = CountingEventSink::default();
    let mut count: u64 = 0;
    let mut shutting_down = false;

    loop {
        if let Some(cmd) = consumer.try_pop() {
            dispatch(&mut book, cmd, &mut events);
            count += 1;
        } else if done.load(Ordering::Acquire) {
            if shutting_down {
                break;
            }
            // Loop again
            shutting_down = true;
        } else {
            std::hint::spin_loop();
        }
    }

    PipelineResult {
        commands_processed: count,
        events,
        final_best_bid: book.best_bid(),
        final_best_ask: book.best_ask(),
        final_order_count: book.order_count(),
    }
}

fn dispatch(book: &mut impl LimitOrderBook, cmd: OrderCommand, events: &mut impl BookEventSink) {
    match cmd {
        OrderCommand::NewOrder {
            order_id,
            side,
            price,
            qty,
        } => book.add_limit_order(order_id, side, price, qty, events),
        OrderCommand::MarketOrder {
            order_id,
            side,
            qty,
        } => book.add_market_order(order_id, side, qty, events),
        OrderCommand::CancelOrder { order_id } => book.cancel_order(order_id, events),
    }
}
