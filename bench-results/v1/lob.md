
# Limit Order Book (v1) — Latency - Latency Report

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-16T22:49:59Z |
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
| lob_version | v1 |
| orders_per_level | 10 |
| resting_orders | 2000 |

## Results

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 1ns | 1ns | 1ns | 42ns | 42ns | 42ns | 459ns | 5ns | 12ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 208ns | 333ns | 375ns | 375ns | 417ns | 500ns | 18.3μs | 332ns | 81ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 416ns | 625ns | 708ns | 709ns | 791ns | 917ns | 14.9μs | 615ns | 107ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 1ns | 1ns | 1ns | 42ns | 42ns | 42ns | 542ns | 5ns | 12ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 1ns | 1ns | 41ns | 42ns | 42ns | 42ns | 292ns | 5ns | 12ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 1ns | 1ns | 1ns | 42ns | 42ns | 125ns | 2ns | 6ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 1ns | 42ns | 83ns | 84ns | 84ns | 143ns | 11.6μs | 49ns | 73ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 1ns | 1ns | 1ns | 41ns | 42ns | 6.8μs | 1ns | 22ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 1ns | 42ns | 42ns | 42ns | 59ns | 666ns | 16ns | 20ns | 0.0 | 0.0 | 0B |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (sustained mix) | 74158657 | 0.0 | 0.0 | 0B | 2 | 11445.0MiB |

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



## Comparison vs Baseline

| Property | Value |
|----------|-------|
| Baseline | "Limit Order Book (v0) — Latency" (2026-03-16T22:49:00Z) |
| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 1ns (↓97.6%) | 42ns (↓50.0%) | 42ns (↓66.4%) | 5ns (↓86.7%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 333ns (↓70.4%) | 417ns (↓69.7%) | 500ns (↓68.4%) | 332ns (↓70.7%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 625ns (↓72.7%) | 791ns (↓71.7%) | 917ns (↓71.6%) | 615ns (↓73.0%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 1ns (↓97.6%) | 42ns (=) | 42ns (↓50.0%) | 5ns (↓84.6%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 1ns (↓99.4%) | 42ns (↓79.9%) | 42ns (↓83.2%) | 5ns (↓96.8%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 1ns (=) | 42ns (=) | 42ns (=) | 2ns (↓15.7%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 42ns (↑2.4%) | 84ns (↑100.0%) | 143ns (↑70.2%) | 49ns (↑61.9%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 1ns (=) | 41ns (↓2.4%) | 42ns (=) | 1ns (↓68.7%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 1ns (↓97.6%) | 42ns (↓50.0%) | 59ns (↓52.8%) | 16ns (↓60.0%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

### Throughput

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (sustained mix) | 74.2M (↑240.6%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 2.0 (↓99.7%) | 11445.0MiB (↑2346238.8%) |

