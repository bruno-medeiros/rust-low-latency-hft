
# Limit Order Book (v1)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-20T20:10:28Z |
| CPU | Apple M4 Pro |
| Cores | 12 |
| Memory | 24.0 GB |
| OS | Darwin 15.7.4 (aarch64) |
| Host | Mac.mynet |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | OS clock (platform fallback via quanta) |
| Baseline | "Limit Order Book (v0)" (2026-03-20T20:08:49Z) |

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
| Add (passive) | 1ns | 41ns | 42ns | 42ns | 42ns | 208ns | 9.1μs | 25ns | 39ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 392ns | 500ns | 542ns | 542ns | 666ns | 750ns | 19.5μs | 512ns | 206ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 792ns | 959ns | 1.1μs | 1.1μs | 1.3μs | 2.4μs | 58.8μs | 992ns | 327ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 1ns | 1ns | 42ns | 83ns | 84ns | 208ns | 3.4μs | 23ns | 32ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 1ns | 1ns | 42ns | 42ns | 83ns | 459ns | 38.1μs | 14ns | 158ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 1ns | 1ns | 1ns | 42ns | 42ns | 209ns | 2ns | 7ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 1ns | 42ns | 84ns | 84ns | 125ns | 375ns | 27.6μs | 57ns | 100ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 1ns | 1ns | 42ns | 42ns | 42ns | 583ns | 4ns | 12ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 42ns | 83ns | 84ns | 84ns | 125ns | 7.9μs | 38ns | 43ns | 0.0 | 0.0 | 0B |

#### vs baseline

| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 41ns (↓2.4%) | 42ns (↓50.0%) | 208ns (↑66.4%) | 25ns (↓37.0%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 500ns (↓53.8%) | 666ns (↓55.6%) | 750ns (↓84.6%) | 512ns (↓53.8%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 959ns (↓56.6%) | 1.3μs (↓54.3%) | 2.4μs (↓50.0%) | 992ns (↓55.4%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 1ns (↓97.6%) | 84ns (↑100.0%) | 208ns (=) | 23ns (↓20.1%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 1ns (↓99.4%) | 83ns (↓66.8%) | 459ns (↑10.3%) | 14ns (↓91.2%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 1ns (=) | 42ns (=) | 42ns (=) | 2ns (↓61.9%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 42ns (↑2.4%) | 125ns (↑50.6%) | 375ns (↑124.6%) | 57ns (↑84.1%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 1ns (=) | 42ns (=) | 42ns (↓49.4%) | 4ns (↓3.5%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 42ns (=) | 84ns (=) | 125ns (↓50.0%) | 38ns (↓11.5%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 40024830 | 0.0 | 0.0 | 0B | 3 | 1.9MiB |

#### vs baseline

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 40.0M (↑47.4%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 3.0 (↓99.5%) | 1.9MiB (↑298.6%) |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116000000 | 0 | 32000000 | 40000000 | 76000000 |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


