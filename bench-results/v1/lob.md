
# Limit Order Book (v1)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-24T17:01:32Z |
| CPU | AMD Ryzen 7 7800X3D 8-Core Processor |
| Cores | 16 |
| Memory | 30.5 GB |
| OS | Linux Mint 22.3 (x86_64) |
| Host | mint |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | TSC (RDTSC via quanta) |
| ASLR | disabled (randomize_va_space=0) |
| CPU governor | performance (all 16 CPUs) |
| IRQ affinity (sample) | mixed (64 sampled IRQs; first=0-15) |
| Isolated CPUs | 2-3 |
| Swap | none active (/proc/swaps header only) |
| Turbo / boost | disabled (AMD cpufreq boost=0) |
| Baseline | "Limit Order Book (v0)" (2026-03-24T16:57:40Z) |

## Latency

| Property | Value |
|----------|-------|
| BENCH_ITERS | 100000 |
| Default pinned core | Could not pin core 2 |
| WARMUP_ITERS | 10000 |
| book_levels | 100 |
| orders_per_level | 10 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 40ns | 60ns | 70ns | 70ns | 80ns | 430ns | 5.7μs | 65ns | 40ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 791ns | 861ns | 891ns | 911ns | 941ns | 3.7μs | 20.7μs | 867ns | 206ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 1.5μs | 1.6μs | 1.7μs | 1.7μs | 1.8μs | 5.5μs | 21.9μs | 1.6μs | 262ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 30ns | 50ns | 60ns | 70ns | 80ns | 300ns | 3.6μs | 52ns | 21ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 20ns | 50ns | 50ns | 60ns | 70ns | 310ns | 3.7μs | 49ns | 22ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 10ns | 50ns | 190ns | 8ns | 4ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 110ns | 160ns | 180ns | 180ns | 190ns | 941ns | 16.6μs | 165ns | 77ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 10ns | 20ns | 20ns | 20ns | 100ns | 851ns | 13ns | 7ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 70ns | 90ns | 90ns | 100ns | 440ns | 17.1μs | 69ns | 74ns | 0.0 | 0.0 | 0B |

#### vs baseline

| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 60ns (↑20.0%) | 80ns (↓20.0%) | 430ns (↑186.7%) | 65ns (↑22.8%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 861ns (↓41.1%) | 941ns (↓43.1%) | 3.7μs (↓26.8%) | 867ns (↓41.5%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 1.6μs (↓43.5%) | 1.8μs (↓46.8%) | 5.5μs (↓19.9%) | 1.6μs (↓43.4%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 50ns (=) | 80ns (↑33.3%) | 300ns (↑66.7%) | 52ns (↑9.7%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 50ns (↓68.8%) | 70ns (↓61.1%) | 310ns (↑72.2%) | 49ns (↓70.0%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 10ns (=) | 10ns (↓50.0%) | 50ns (↑150.0%) | 8ns (↓9.4%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 160ns (↑166.7%) | 190ns (↑137.5%) | 941ns (↑104.6%) | 165ns (↑155.3%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 10ns (↓50.0%) | 20ns (↓33.3%) | 100ns (↓23.1%) | 13ns (↓28.7%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 70ns (↑16.7%) | 100ns (↓9.1%) | 440ns (↑83.3%) | 69ns (↑11.6%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| Default pinned core | Could not pin core 2 |
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 44.3M | 0.0 | 0.0 | 0B | 3 | 1.9MiB |

#### vs baseline

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 44.3M (↑71.8%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 3.0 (↓99.5%) | 1.9MiB (↑298.5%) |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


