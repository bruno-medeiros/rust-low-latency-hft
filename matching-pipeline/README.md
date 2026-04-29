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

