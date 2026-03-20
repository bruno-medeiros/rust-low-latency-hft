//! Integration test: run the full pipeline against a real LOBSTER sample file
//! (GOOG 2012-06-21, level 1).

use limit_order_book::LimitOrderBookV1;
use matching_pipeline::{LobsterParser, Pipeline, PipelineConfig};

const SAMPLE_FILE: &str = "LOBSTER_SampleFiles/GOOG_2012-06-21_34200000_57600000_message_1.csv";

fn load_sample() -> String {
    let path = format!("{}/{SAMPLE_FILE}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"))
}

#[test]
fn extracts_expected_command_count() {
    let csv = load_sample();
    let parser = LobsterParser::new();
    let rows = parser.parse_messages(&csv).unwrap();
    let commands = parser.extract_commands(&rows);

    // type 1 (new order): 24 368 + type 3 (full cancel): 13 429
    assert_eq!(commands.len(), 37_797);
}

#[test]
fn pipeline_completes_with_consistent_book() {
    let csv = load_sample();
    let parser = LobsterParser::new();
    let rows = parser.parse_messages(&csv).unwrap();
    let commands = parser.extract_commands(&rows);
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
