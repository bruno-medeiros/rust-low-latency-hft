
# Limit Order Book (v1)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-24T12:47:28Z |
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
| Baseline | "Limit Order Book (v0)" (2026-03-24T12:43:25Z) |

## Latency

| Property | Value |
|----------|-------|
| BENCH_ITERS | 100000 |
| Default pinned core | pin core 2 |
| WARMUP_ITERS | 10000 |
| book_levels | 100 |
| orders_per_level | 10 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 40ns | 60ns | 70ns | 70ns | 80ns | 450ns | 661ns | 61ns | 20ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 781ns | 851ns | 891ns | 901ns | 1.1μs | 1.6μs | 2.8μs | 856ns | 60ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 1.5μs | 1.6μs | 1.7μs | 1.7μs | 1.8μs | 2.6μs | 4.7μs | 1.6μs | 70ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 30ns | 50ns | 60ns | 60ns | 80ns | 290ns | 470ns | 51ns | 15ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 20ns | 50ns | 60ns | 70ns | 80ns | 310ns | 490ns | 54ns | 16ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 10ns | 50ns | 160ns | 8ns | 3ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 110ns | 160ns | 170ns | 180ns | 220ns | 1.1μs | 1.6μs | 163ns | 49ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 10ns | 20ns | 20ns | 20ns | 80ns | 320ns | 12ns | 5ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 70ns | 90ns | 90ns | 110ns | 430ns | 781ns | 68ns | 26ns | 0.0 | 0.0 | 0B |

#### vs baseline

| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 60ns (↑20.0%) | 80ns (↓20.0%) | 450ns (↑164.7%) | 61ns (↑8.7%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 851ns (↓41.4%) | 1.1μs (↓34.5%) | 1.6μs (↓31.8%) | 856ns (↓41.4%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 1.6μs (↓43.4%) | 1.8μs (↓46.2%) | 2.6μs (↓43.4%) | 1.6μs (↓43.2%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 50ns (↑25.0%) | 80ns (↑33.3%) | 290ns (↑81.2%) | 51ns (↑21.6%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 50ns (↓68.8%) | 80ns (↓52.9%) | 310ns (↑29.2%) | 54ns (↓65.9%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 10ns (=) | 10ns (↓50.0%) | 50ns (↑66.7%) | 8ns (↓11.0%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 160ns (↑300.0%) | 220ns (↑214.3%) | 1.1μs (↑282.9%) | 163ns (↑278.4%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 10ns (↓50.0%) | 20ns (↓33.3%) | 80ns (↑33.3%) | 12ns (↓38.8%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 70ns (↑16.7%) | 110ns (=) | 430ns (↑79.2%) | 68ns (↑14.0%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| Default pinned core | pin core 2 |
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 44.0M | 0.0 | 0.0 | 0B | 3 | 1.9MiB |

#### vs baseline

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 44.0M (↑73.9%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 3.0 (↓99.5%) | 1.9MiB (↑298.5%) |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


