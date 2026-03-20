
# Limit Order Book (v0) — Latency - Latency Report

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-16T22:49:00Z |
| CPU | Apple M4 Pro |
| Cores | 12 |
| Memory | 24.0 GB |
| OS | Darwin 15.7.4 (aarch64) |
| Host | Mac.mynet |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | OS clock (platform fallback via quanta) |
| Samples | 100000 (warmup: 10000) |
| book_levels | 100 |
| crowded_level_orders | 500 |
| iters | 100000 |
| lob_version | v0 |
| orders_per_level | 10 |
| resting_orders | 2000 |

## Results

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 1ns | 42ns | 42ns | 83ns | 84ns | 125ns | 12.7μs | 38ns | 48ns | 1.0 | 0.0 | 32B |
| Add (sweep 5 levels, 50 fills) | 833ns | 1.1μs | 1.2μs | 1.2μs | 1.4μs | 1.6μs | 30.8μs | 1.1μs | 179ns | 0.0 | 6.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 1.8μs | 2.3μs | 2.4μs | 2.5μs | 2.8μs | 3.2μs | 24.5μs | 2.3μs | 232ns | 0.0 | 14.0 | 0B |
| Cancel (head of queue) | 1ns | 42ns | 42ns | 42ns | 42ns | 84ns | 3.2μs | 32ns | 21ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 100ns | 167ns | 208ns | 209ns | 209ns | 250ns | 6.2μs | 172ns | 33ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 1ns | 1ns | 1ns | 42ns | 42ns | 292ns | 2ns | 7ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 1ns | 41ns | 42ns | 42ns | 42ns | 84ns | 458ns | 30ns | 19ns | 1.0 | 1.0 | 80B |
| Order lookup (hit) | 1ns | 1ns | 1ns | 42ns | 42ns | 42ns | 333ns | 4ns | 12ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 42ns | 83ns | 84ns | 84ns | 125ns | 3.4μs | 42ns | 31ns | 0.4 | 0.0 | 13B |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (sustained mix) | 21772709 | 38.0 | 35.0 | 3.3KiB | 645 | 499.5KiB |

#### Throughput (sustained mix) — event counts

| Event type | Count |
|------------|-------|
| Accepted | 116000000 |
| Rejected | 0 |
| Fill | 32000000 |
| Filled | 40000000 |
| Cancelled | 76000000 |

#### Throughput flamegraph

![Flame graph](flamegraph.svg)


