
# Limit Order Book (v0)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-20T19:12:26Z |
| CPU | Apple M4 Pro |
| Cores | 12 |
| Memory | 24.0 GB |
| OS | Darwin 15.7.4 (aarch64) |
| Host | Mac.mynet |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | OS clock (platform fallback via quanta) |

## Latency

| Property | Value |
|----------|-------|
| BENCH_ITERS | 100000 |
| WARMUP_ITERS | 10000 |
| book_levels | 100 |
| orders_per_level | 10 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 1ns | 42ns | 42ns | 83ns | 84ns | 125ns | 18.9μs | 39ns | 68ns | 1.0 | 0.0 | 32B |
| Add (sweep 5 levels, 50 fills) | 851ns | 1.1μs | 1.2μs | 1.3μs | 1.5μs | 4.8μs | 58.0μs | 1.1μs | 434ns | 0.0 | 6.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 1.8μs | 2.3μs | 2.4μs | 2.5μs | 2.8μs | 3.8μs | 42.1μs | 2.2μs | 318ns | 0.0 | 14.0 | 0B |
| Cancel (head of queue) | 1ns | 41ns | 42ns | 42ns | 42ns | 84ns | 5.5μs | 29ns | 26ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 100ns | 167ns | 208ns | 208ns | 209ns | 250ns | 21.7μs | 166ns | 87ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 1ns | 1ns | 1ns | 42ns | 42ns | 167ns | 2ns | 8ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 1ns | 42ns | 42ns | 42ns | 84ns | 333ns | 19.6μs | 35ns | 104ns | 1.0 | 1.0 | 80B |
| Order lookup (hit) | 1ns | 1ns | 1ns | 42ns | 42ns | 42ns | 333ns | 4ns | 11ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 42ns | 83ns | 84ns | 84ns | 166ns | 5.2μs | 43ns | 36ns | 0.4 | 0.0 | 13B |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 28428211 | 38.0 | 35.0 | 3.3KiB | 645 | 499.5KiB |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116000000 | 0 | 32000000 | 40000000 | 76000000 |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


