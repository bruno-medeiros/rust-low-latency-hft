//! Integration test: run the full pipeline against a real LOBSTER sample file
//! (GOOG 2012-06-21, level 1).

use limit_order_book::LimitOrderBookV1;
use matching_pipeline::{Pipeline, PipelineConfig, test_data};

#[test]
fn extracts_expected_command_count() {
    let commands = test_data::goog_sample_commands();

    // type 1 (new order): 24 368 + type 3 (full cancel): 13 429
    assert_eq!(commands.len(), 37_797);
}

#[test]
fn pipeline_completes_with_consistent_book() {
    let commands = test_data::goog_sample_commands().to_vec();
    let num_commands = commands.len() as u64;

    let config = PipelineConfig {
        queue_slots: 4096,
        price_range: (5_500_000, 5_900_000),
        order_capacity: 30_000,
    };

    let result = Pipeline::new(config).run::<LimitOrderBookV1>(commands);

    assert_eq!(result.commands_processed, num_commands);
    assert!(result.events.accepted > 0);
    assert!(
        result.events.fill > 0,
        "expected at least one fill from crossing orders"
    );

    eprintln!(
        "GOOG pipeline: {} commands -> {} accepted, {} rejected, {} fills, {} filled, {} cancelled | \
         book: {} resting orders",
        result.commands_processed,
        result.events.accepted,
        result.events.rejected,
        result.events.fill,
        result.events.filled,
        result.events.cancelled,
        result.final_order_count,
    );
}
