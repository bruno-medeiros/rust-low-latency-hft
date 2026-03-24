
# Limit Order Book (v1)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-24T15:29:29Z |
| CPU | AMD Ryzen 7 7800X3D 8-Core Processor |
| Cores | 16 |
| Memory | 30.5 GB |
| OS | Linux Mint 22.3 (x86_64) |
| Host | mint |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | TSC (RDTSC via quanta) |
| ASLR | enabled full (randomize_va_space=2) |
| CPU governor | powersave (all 16 CPUs) |
| IRQ affinity (sample) | mixed (64 sampled IRQs; first=0-15) |
| Isolated CPUs | 2-3 |
| Swap | none active (/proc/swaps header only) |
| Turbo / boost | enabled (AMD cpufreq boost=1) |
| Baseline | "Limit Order Book (v0)" (2026-03-24T15:26:12Z) |

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
| Add (passive) | 20ns | 50ns | 50ns | 60ns | 60ns | 460ns | 4.3μs | 49ns | 33ns | 0.0 | 0.0 | 0B |
| Add (sweep 5 levels, 50 fills) | 651ns | 691ns | 721ns | 731ns | 771ns | 1.6μs | 13.9μs | 701ns | 131ns | 0.0 | 0.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 1.3μs | 1.3μs | 1.4μs | 1.4μs | 1.5μs | 4.4μs | 14.9μs | 1.3μs | 187ns | 0.0 | 0.0 | 0B |
| Cancel (head of queue) | 30ns | 40ns | 50ns | 50ns | 60ns | 360ns | 2.8μs | 42ns | 20ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 20ns | 40ns | 40ns | 50ns | 50ns | 350ns | 10.4μs | 40ns | 47ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 10ns | 70ns | 270ns | 7ns | 5ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 90ns | 130ns | 140ns | 150ns | 160ns | 951ns | 11.4μs | 133ns | 76ns | 2.0 | 1.0 | 128B |
| Order lookup (hit) | 1ns | 10ns | 20ns | 20ns | 20ns | 90ns | 9.5μs | 11ns | 30ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 60ns | 70ns | 80ns | 90ns | 440ns | 10.5μs | 58ns | 44ns | 0.0 | 0.0 | 0B |

#### vs baseline

| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-------|------|-----------|-------------|----------|
| Add (passive) | 50ns (↑25.0%) | 60ns (↓25.0%) | 460ns (↑130.0%) | 49ns (↑6.5%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |
| Add (sweep 5 levels, 50 fills) | 691ns (↓43.5%) | 771ns (↓42.5%) | 1.6μs (↓58.0%) | 701ns (↓43.1%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Market (sweep 10 levels, 100 fills) | 1.3μs (↓45.1%) | 1.5μs (↓48.1%) | 4.4μs (↓19.3%) | 1.3μs (↓45.0%) | 0.0 (=) | 0.0 (↓100.0%) | 0B (=) |
| Cancel (head of queue) | 40ns (=) | 60ns (↑20.0%) | 360ns (↑20.0%) | 42ns (↑7.6%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Cancel (tail of queue) | 40ns (↓73.3%) | 50ns (↓68.8%) | 350ns (↑105.9%) | 40ns (↓72.8%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Spread (BBO query) | 10ns (=) | 10ns (↓50.0%) | 70ns (↑250.0%) | 7ns (↓6.0%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Depth (top 5) | 130ns (↑225.0%) | 160ns (↑220.0%) | 951ns (↑150.3%) | 133ns (↑209.9%) | 2.0 (↑100.0%) | 1.0 (=) | 128B (↑60.0%) |
| Order lookup (hit) | 10ns (↓50.0%) | 20ns (↓33.3%) | 90ns (↓57.1%) | 11ns (↓51.6%) | 0.0 (=) | 0.0 (=) | 0B (=) |
| Realistic mix (per-op) | 60ns (↑20.0%) | 90ns (↓10.0%) | 440ns (↑37.5%) | 58ns (↑5.0%) | 0.0 (↓100.0%) | 0.0 (=) | 0B (↓100.0%) |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| Default pinned core | Could not pin core 2 |
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 52.8M | 0.0 | 0.0 | 0B | 3 | 1.9MiB |

#### vs baseline

| Operation | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|-----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 52.8M (↑124.6%) | 0.0 (↓100.0%) | 0.0 (↓100.0%) | 0B (↓100.0%) | 3.0 (↓99.5%) | 1.9MiB (↑298.5%) |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


