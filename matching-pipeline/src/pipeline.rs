use core_affinity::CoreId;
use limit_order_book::LimitOrderBook;
use limit_order_book::event::CountingEventSink;
use limit_order_book::types::{Price, Qty};
use lockfree_queue::spsc::SpscQueue;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

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
    /// Whether to pin threads to cores
    pub core_pinning_enabled: bool,
    /// CPU core for the producer thread (`ingest_commands`);
    pub producer_pin_core: u32,
    /// CPU core for the consumer / matching thread;
    pub consumer_pin_core: u32,
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
    pub done: Arc<AtomicBool>,
    pub consumer_handle: thread::JoinHandle<PipelineResult>,
    pub producer: lockfree_queue::spsc::SpscProducer<OrderCommand>,
}

impl Pipeline {
    pub fn new<B: LimitOrderBook + Send + 'static>(config: PipelineConfig) -> Self {
        let price_range = config.price_range;
        let order_capacity = config.order_capacity;
        let book = B::with_config(price_range, order_capacity);

        let queue = SpscQueue::new(config.queue_slots).expect("invalid queue slot count");
        let (producer, consumer) = queue.split();
        let done = Arc::new(AtomicBool::new(false));

        if config.core_pinning_enabled {
            let id = config.producer_pin_core as usize;
            core_affinity::set_for_current(CoreId { id });
        }

        let done_rx = done.clone();
        let consumer_handle = thread::spawn(move || {
            if config.core_pinning_enabled {
                let id = config.consumer_pin_core as usize;
                core_affinity::set_for_current(CoreId { id });
            }
            consumer::consume::<B>(consumer, done_rx, book)
        });

        Self {
            done,
            consumer_handle,
            producer,
        }
    }

    pub fn ingest_commands(&mut self, commands: &[OrderCommand]) {
        for &cmd in commands {
            self.producer.push_blocking(cmd);
        }
    }

    pub fn run_and_terminate(mut self, commands: &[OrderCommand]) -> PipelineResult {
        self.ingest_commands(commands);
        self.done.store(true, Ordering::Release);
        self.consumer_handle.join().unwrap()
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

        let result = test_pipeline().run_and_terminate(&commands);

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

        let result = test_pipeline().run_and_terminate(&commands);

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

        let result = test_pipeline().run_and_terminate(&commands);

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

        let result = test_pipeline().run_and_terminate(&commands);

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

        let result = test_pipeline().run_and_terminate(&commands);

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

        let result = test_pipeline().run_and_terminate(&commands);

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

        let pipeline = Pipeline::new::<LimitOrderBookV1>(PipelineConfig {
            queue_slots: 64,
            price_range: (1, 10_000),
            order_capacity: 100,
            core_pinning_enabled: false,
            producer_pin_core: 2,
            consumer_pin_core: 3,
        });

        let result = pipeline.run_and_terminate(&commands);

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
        let result = test_pipeline().run_and_terminate(&Vec::new());

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

        let result = test_pipeline().run_and_terminate(&commands);

        assert_eq!(result.events.accepted, 1);
        assert_eq!(result.events.rejected, 1);
        assert_eq!(result.final_order_count, 1);
    }

    fn test_pipeline() -> Pipeline {
        Pipeline::new::<LimitOrderBookV1>(PipelineConfig {
            queue_slots: 64,
            price_range: (1, 10_000),
            order_capacity: 1_000,
            core_pinning_enabled: true,
            producer_pin_core: 2,
            consumer_pin_core: 3,
        })
    }
}
