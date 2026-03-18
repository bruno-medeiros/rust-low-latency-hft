# Low Latency / HFT demos

A portfolio repository to showcase low-latency / HFT projects, with latency and throughput benchmarks to demonstrate the achieved performance.

## Projects

### [Limit Order Book](limit-order-book/)

- a high-performance limit order book engine optimised for low-latency matching, with nanosecond-level benchmarks.
- **p99 add: 42ns | p99 cancel: 42ns | throughput: 74M ops/sec | zero heap allocations on the hot path**
- Full results: [Benchmark report](limit-order-book/bench-results/v1/lob.md) — includes latency percentiles (min → p99.9), throughput, allocation tracking, flamegraph, and comparison vs v0 (baseline).

*TODO: re-run benchmarks in Linux*

## Projects TODO:

## Additional Notes

See [Benchmark Methodology](Benchmark-Methodology.md) for timing methodology, measurement techniques, etc.