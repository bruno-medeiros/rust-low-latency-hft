use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use limit_order_book::LimitOrderBook;
use limit_order_book::event::CountingEventSink;
use limit_order_book::types::{Price, Qty};
use lockfree_queue::spsc::SpscQueue;

use crate::command::OrderCommand;
use crate::consumer;

#[derive(Clone, Copy)]
pub struct PipelineConfig {
    /// SPSC queue ring size (must be a power of two, ≥ 2).
    pub queue_slots: usize,
    /// LOB price tick range `(min, max)`.
    pub price_range: (Price, Price),
    /// LOB pre-allocation hint for order capacity.
    pub order_capacity: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            queue_slots: 4096,
            price_range: (1, 100_000),
            order_capacity: 10_000,
        }
    }
}

/// Aggregate results returned after the pipeline drains all commands.
pub struct PipelineResult {
    pub commands_processed: u64,
    pub events: CountingEventSink,
    pub final_best_bid: Option<(Price, Qty)>,
    pub final_best_ask: Option<(Price, Qty)>,
    pub final_order_count: u64,
}

/// Two-thread matching-engine pipeline.
pub struct Pipeline {
    config: PipelineConfig,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Run the pipeline: spawn producer and consumer threads, push all commands
    /// through the SPSC queue, match in the LOB, and return once both threads
    /// have joined.
    pub fn run<B: LimitOrderBook + Send + 'static>(
        &self,
        // REVIEW: take an iterator of OrderCommand
        commands: Vec<OrderCommand>,
    ) -> PipelineResult {
        let queue = SpscQueue::new(self.config.queue_slots).expect("invalid queue slot count");
        let (mut producer, consumer) = queue.split();

        let done = Arc::new(AtomicBool::new(false));
        let done_rx = done.clone();

        let price_range = self.config.price_range;
        let order_capacity = self.config.order_capacity;

        let consumer_handle = thread::spawn(move || {
            let book = B::with_config(price_range, order_capacity);
            consumer::consume::<B>(consumer, done_rx, book)
        });

        let producer_handle = thread::spawn(move || {
            for cmd in commands {
                producer.push_blocking(cmd);
            }
            done.store(true, Ordering::Release);
        });

        producer_handle.join().expect("producer thread panicked");
        consumer_handle.join().expect("consumer thread panicked")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobster::LobsterParser;
    use limit_order_book::LimitOrderBookV1;
    use limit_order_book::types::Side;

    // --- basic matching ---

    #[test]
    fn full_fill_clears_both_orders() {
        let commands = vec![
            OrderCommand::NewOrder {
                order_id: 1,
                side: Side::Buy,
                price: 100,
                qty: 10,
            },
            OrderCommand::NewOrder {
                order_id: 2,
                side: Side::Sell,
                price: 100,
                qty: 10,
            },
        ];

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.commands_processed, 2);
        assert_eq!(result.events.accepted, 2);
        assert_eq!(result.events.fill, 1);
        assert_eq!(result.events.filled, 2); // both aggressor + passive
        assert_eq!(result.final_best_bid, None);
        assert_eq!(result.final_best_ask, None);
        assert_eq!(result.final_order_count, 0);
    }

    #[test]
    fn partial_fill_leaves_remainder() {
        let commands = vec![
            OrderCommand::NewOrder {
                order_id: 1,
                side: Side::Buy,
                price: 100,
                qty: 10,
            },
            OrderCommand::NewOrder {
                order_id: 2,
                side: Side::Sell,
                price: 100,
                qty: 3,
            },
        ];

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.events.fill, 1);
        assert_eq!(result.events.filled, 1); // only the sell is fully filled
        assert_eq!(result.final_best_bid, Some((100, 7)));
        assert_eq!(result.final_best_ask, None);
        assert_eq!(result.final_order_count, 1);
    }

    // --- cancel ---

    #[test]
    fn cancel_removes_resting_order() {
        let commands = vec![
            OrderCommand::NewOrder {
                order_id: 1,
                side: Side::Buy,
                price: 100,
                qty: 10,
            },
            OrderCommand::NewOrder {
                order_id: 2,
                side: Side::Sell,
                price: 200,
                qty: 10,
            },
            OrderCommand::CancelOrder { order_id: 1 },
        ];

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.events.accepted, 2);
        assert_eq!(result.events.cancelled, 1);
        assert_eq!(result.events.fill, 0);
        assert_eq!(result.final_best_bid, None);
        assert_eq!(result.final_best_ask, Some((200, 10)));
        assert_eq!(result.final_order_count, 1);
    }

    #[test]
    fn cancel_unknown_order_emits_reject() {
        let commands = vec![OrderCommand::CancelOrder { order_id: 999 }];

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.commands_processed, 1);
        assert_eq!(result.events.rejected, 1);
    }

    // --- market order ---

    #[test]
    fn market_order_sweeps_resting_liquidity() {
        let commands = vec![
            OrderCommand::NewOrder {
                order_id: 1,
                side: Side::Sell,
                price: 100,
                qty: 5,
            },
            OrderCommand::NewOrder {
                order_id: 2,
                side: Side::Sell,
                price: 101,
                qty: 5,
            },
            OrderCommand::MarketOrder {
                order_id: 3,
                side: Side::Buy,
                qty: 8,
            },
        ];

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.events.fill, 2); // matches at price 100 and 101
        assert_eq!(result.events.filled, 2); // sell@100 fully filled, market order fully filled
        assert_eq!(result.final_best_ask, Some((101, 2)));
        assert_eq!(result.final_order_count, 1);
    }

    // --- crossing boundary ---

    #[test]
    fn crossing_orders_match_at_boundary() {
        let mut commands = Vec::new();
        let mut id = 1u64;

        for price in 90..=100 {
            commands.push(OrderCommand::NewOrder {
                order_id: id,
                side: Side::Buy,
                price,
                qty: 10,
            });
            id += 1;
        }
        for price in 100..=110 {
            commands.push(OrderCommand::NewOrder {
                order_id: id,
                side: Side::Sell,
                price,
                qty: 10,
            });
            id += 1;
        }

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.commands_processed, 22);
        assert_eq!(result.events.accepted, 22);
        assert_eq!(result.events.fill, 1);
        assert_eq!(result.events.filled, 2);
        assert_eq!(result.final_best_bid, Some((99, 10)));
        assert_eq!(result.final_best_ask, Some((101, 10)));
        assert_eq!(result.final_order_count, 20);
    }

    // --- LOBSTER replay ---

    #[test]
    fn lobster_replay_maintains_consistency() {
        let csv = "\
34200.0,1,1,100,5000,1
34200.0,1,2,100,4900,1
34200.0,1,3,100,5100,-1
34200.0,1,4,100,5200,-1
34200.0,3,2,100,4900,1
";
        let parser = LobsterParser::new();
        let rows = parser.parse_messages(csv).unwrap();
        let commands = parser.extract_commands(&rows);

        let pipeline = Pipeline::new(PipelineConfig {
            queue_slots: 64,
            price_range: (1, 10_000),
            order_capacity: 100,
        });

        let result = pipeline.run::<LimitOrderBookV1>(commands);

        assert_eq!(result.final_best_bid, Some((5000, 100)));
        assert_eq!(result.final_best_ask, Some((5100, 100)));
        assert_eq!(result.final_order_count, 3);
        assert_eq!(result.events.accepted, 4);
        assert_eq!(result.events.cancelled, 1);
        assert_eq!(result.events.fill, 0);
    }

    // --- edge cases ---

    #[test]
    fn empty_command_list() {
        let result = test_pipeline().run::<LimitOrderBookV1>(vec![]);

        assert_eq!(result.commands_processed, 0);
        assert_eq!(result.final_best_bid, None);
        assert_eq!(result.final_best_ask, None);
        assert_eq!(result.final_order_count, 0);
    }

    #[test]
    fn duplicate_order_id_rejected() {
        let commands = vec![
            OrderCommand::NewOrder {
                order_id: 1,
                side: Side::Buy,
                price: 100,
                qty: 10,
            },
            OrderCommand::NewOrder {
                order_id: 1,
                side: Side::Sell,
                price: 200,
                qty: 5,
            },
        ];

        let result = test_pipeline().run::<LimitOrderBookV1>(commands);

        assert_eq!(result.events.accepted, 1);
        assert_eq!(result.events.rejected, 1);
        assert_eq!(result.final_order_count, 1);
    }

    fn test_pipeline() -> Pipeline {
        Pipeline::new(PipelineConfig {
            queue_slots: 64,
            price_range: (1, 10_000),
            order_capacity: 1_000,
        })
    }
}
