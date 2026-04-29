
# Limit Order Book (v1)

| Property | Value |
|----------|-------|
| Timestamp | 2026-04-29T14:43:28Z |
| CPU | AMD Ryzen 7 7800X3D 8-Core Processor |
| Cores | 16 |
| Memory | 30.5 GB |
| OS | Linux Mint 22.3 (x86_64) |
| Host | bruno-linux |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | TSC (RDTSC via quanta) |
| ASLR | disabled (randomize_va_space=0) |
| CPU governor | performance (all 16 CPUs) |
| IRQ affinity (sample) | mixed (64 sampled IRQs; first=0-15) |
| Isolated CPUs | 2-3,10-11 |
| Swap | none active (/proc/swaps header only) |
| Turbo / boost | disabled (AMD cpufreq boost=0) |
| Baseline | "Limit Order Book (v0)" (2026-04-29T13:40:39Z) |

## Latency

| Property | Value |
|----------|-------|
| book_levels | 100 |
| orders_per_level | 10 |
| BENCH_ITERS | 100000 |
| WARMUP_ITERS | 10000 |
| Default pinned core | pin core 2 |

### Latency

| Operation | min | p50 | p90 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 30ns | 40ns | 50ns | 60ns | 350ns | 561ns | 45ns | 20ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 781ns | 831ns | 871ns | 941ns | 1.6μs | 2.1μs | 836ns | 57ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 1.5μs | 1.6μs | 1.7μs | 1.8μs | 2.4μs | 2.9μs | 1.6μs | 59ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 30ns | 40ns | 60ns | 80ns | 320ns | 500ns | 47ns | 19ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 20ns | 40ns | 50ns | 60ns | 350ns | 520ns | 42ns | 21ns | 0.0 | 0.0 | 0B |
| Reduce (partial) | 20ns | 30ns | 40ns | 50ns | 150ns | 360ns | 35ns | 9ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 40ns | 250ns | 8ns | 3ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 150ns | 180ns | 190ns | 210ns | 1.2μs | 2.5μs | 182ns | 52ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 10ns | 20ns | 20ns | 120ns | 440ns | 12ns | 7ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 70ns | 80ns | 90ns | 420ns | 701ns | 63ns | 25ns | 0.0 | 0.0 | 0B |

#### vs baseline

| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 40ns (↓20.0%) | 60ns (↓33.3%) | 350ns (↑191.7%) | 45ns (↓10.8%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 831ns (↓43.2%) | 941ns (↓42.7%) | 1.6μs (↓32.7%) | 836ns (↓43.3%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 1.6μs (↓44.7%) | 1.8μs (↓47.5%) | 2.4μs (↓41.6%) | 1.6μs (↓45.0%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 40ns (↓20.0%) | 80ns (=) | 320ns (↓27.3%) | 47ns (↓14.3%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 40ns (↓75.0%) | 60ns (↓64.7%) | 350ns (↑94.4%) | 42ns (↓73.0%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Reduce (partial) | 30ns (=) | 50ns (↑25.0%) | 150ns (↑200.0%) | 35ns (↑7.3%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 10ns (=) | 10ns (↓50.0%) | 40ns (↓63.6%) | 8ns (↓4.5%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 180ns (↑350.0%) | 210ns (↑250.0%) | 1.2μs (↑462.9%) | 182ns (↑329.4%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 10ns (=) | 20ns (=) | 120ns (↑300.0%) | 12ns (↑10.1%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 70ns (↑16.7%) | 90ns (↓18.2%) | 420ns (↓10.6%) | 63ns (↓1.5%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| book_levels | 100 |
| orders_per_level | 10 |
| Default pinned core | pin core 2 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 44.3M | 0.0 | 0.0 | 0B | 3 | 1.9MiB |

#### vs baseline

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 44.3M (↑130.7%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 3.0 (↓99.5%) | 1.9MiB (↑298.5%) |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


