# Low Latency / HFT demos

A portfolio of Rust low-latency / HFT demos with reproducible latency/throughput benchmarks.

## Projects

### [Limit Order Book](limit-order-book/README.md)

- a high-performance limit order book engine optimised for low-latency matching, with nanosecond-level benchmarks.
- **p99 add: 42ns | p99 cancel: 42ns | throughput: 74M ops/sec | zero heap allocations on the hot path**
- Full results: [Benchmark report](limit-order-book/bench-results/v1/lob.md) — includes latency percentiles (min → p99.9), throughput, allocation tracking, flamegraph, and comparison vs v0 (baseline).

*TODO: re-run benchmarks in Linux*

### [lockfree-queue](lockfree-queue/README.md)

- SPSC lock-free ring buffer using head + tail indexes.

### [market-data-feed](market-data-feed/README.md)

- ITCH-style binary market data feed handler.


## Additional Notes

See [Benchmark Methodology](Benchmark-Methodology.md) for timing methodology, measurement techniques, etc.