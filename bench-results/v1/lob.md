
# Limit Order Book (v1)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-20T19:14:04Z |
| CPU | Apple M4 Pro |
| Cores | 12 |
| Memory | 24.0 GB |
| OS | Darwin 15.7.4 (aarch64) |
| Host | Mac.mynet |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | OS clock (platform fallback via quanta) |
| Baseline | "Limit Order Book (v0)" (2026-03-20T19:12:26Z) |

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
| Add (passive) | 1ns | 41ns | 42ns | 42ns | 42ns | 166ns | 8.3μs | 26ns | 38ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 392ns | 500ns | 542ns | 584ns | 708ns | 2.3μs | 37.9μs | 498ns | 234ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 808ns | 917ns | 1.0μs | 1.1μs | 1.3μs | 4.0μs | 43.2μs | 966ns | 325ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 1ns | 1ns | 42ns | 42ns | 42ns | 125ns | 7.6μs | 12ns | 38ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 1ns | 1ns | 42ns | 42ns | 42ns | 83ns | 684ns | 10ns | 18ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 1ns | 1ns | 1ns | 42ns | 42ns | 500ns | 2ns | 7ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 1ns | 42ns | 84ns | 84ns | 125ns | 667ns | 15.8μs | 57ns | 81ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 1ns | 1ns | 42ns | 42ns | 42ns | 292ns | 4ns | 11ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 41ns | 42ns | 83ns | 84ns | 333ns | 27.3μs | 31ns | 98ns | 0.0 | 0.0 | 0B |

#### vs baseline

| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 41ns (↓2.4%) | 42ns (↓50.0%) | 166ns (↑32.8%) | 26ns (↓33.9%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 500ns (↓55.6%) | 708ns (↓54.1%) | 2.3μs (↓53.4%) | 498ns (↓56.2%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 917ns (↓59.3%) | 1.3μs (↓52.9%) | 4.0μs (↑5.5%) | 966ns (↓56.9%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 1ns (↓97.6%) | 42ns (=) | 125ns (↑48.8%) | 12ns (↓57.7%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 1ns (↓99.4%) | 42ns (↓79.9%) | 83ns (↓66.8%) | 10ns (↓93.8%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 1ns (=) | 42ns (=) | 42ns (=) | 2ns (↓22.8%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 42ns (=) | 125ns (↑48.8%) | 667ns (↑100.3%) | 57ns (↑60.0%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 1ns (=) | 42ns (=) | 42ns (=) | 4ns (↓3.1%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 41ns (↓2.4%) | 84ns (=) | 333ns (↑100.6%) | 31ns (↓28.5%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 41136792 | 0.0 | 0.0 | 0B | 3 | 1.9MiB |

#### vs baseline

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 41.1M (↑44.7%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 3.0 (↓99.5%) | 1.9MiB (↑298.6%) |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116000000 | 0 | 32000000 | 40000000 | 76000000 |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


