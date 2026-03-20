# matching-pipeline

Two-thread matching-engine pipeline that connects a LOBSTER data source to the limit order book via the lock-free SPSC queue.

```text
LOBSTER CSV ──parse──► Vec<OrderCommand>
                            │
              [producer]    │    [consumer]
                  push ──► SPSC queue ──► LOB matching engine ──► PipelineResult
```

Orders are pre-parsed from a LOBSTER message file (cold path), then replayed through the SPSC queue into the matching engine (hot path). The producer and consumer run on separate threads; the hot path is fully in-memory with no file I/O.

## Crate dependencies

| Crate | Role in the pipeline |
|---|---|
| `limit-order-book` | Matching engine (consumer thread) |
| `lockfree-queue` | SPSC ring buffer connecting producer → consumer |

## Benchmarks

From the repository root, `./run-benchmarks-and-report.sh` runs the `pipeline` throughput benchmark and writes [`report.md`](../bench-results/matching-pipeline/report.md).

Ad hoc:

```bash
cargo bench -p matching-pipeline --bench pipeline
```

## Getting LOBSTER data

LOBSTER provides free sample files derived from NASDAQ TotalView-ITCH data:

1. Go to <https://lobsterdata.com/info/DataSamples.php>
2. Download a **message file** for any stock (AAPL, AMZN, GOOG, INTC, MSFT) — pick any depth level, only the message file is needed
3. Extract the `.7z` archive

Files are CSV with 6 columns: `Time,EventType,OrderID,Size,Price,Direction`. The pipeline uses event types 1 (new limit order) and 3 (full cancel) as order-entry commands; execution reports and other event types are skipped.

## Usage example

```rust
use matching_pipeline::{LobsterParser, OrderCommand, Pipeline, PipelineConfig};
use limit_order_book::LimitOrderBookV1;

// Cold path: parse LOBSTER CSV into order commands
let csv = std::fs::read_to_string("AAPL_2012-06-21_message_5.csv").unwrap();
let parser = LobsterParser::new();
let rows = parser.parse_messages(&csv).unwrap();
let commands = parser.extract_commands(&rows);

// Configure the pipeline
let config = PipelineConfig {
    queue_slots: 65536,
    price_range: (1, 10_000_000),  // LOBSTER prices are USD × 10000
    order_capacity: 500_000,
};

// Hot path: replay through SPSC queue → matching engine
let result = Pipeline::new(config).run::<LimitOrderBookV1>(commands);

assert!(result.book_consistent());
println!(
    "processed {} commands: {} accepted, {} fills, {} cancels",
    result.commands_processed,
    result.events.accepted,
    result.events.fill,
    result.events.cancelled,
);
```
