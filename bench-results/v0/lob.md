
# Limit Order Book (v0)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-24T15:26:12Z |
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
| Add (passive) | 30ns | 40ns | 60ns | 60ns | 80ns | 200ns | 4.1μs | 46ns | 26ns | 1.0 | 0.0 | 32B |
| Add (sweep 5 levels, 50 fills) | 1.1μs | 1.2μs | 1.3μs | 1.3μs | 1.3μs | 3.7μs | 23.1μs | 1.2μs | 266ns | 0.0 | 6.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 2.3μs | 2.4μs | 2.5μs | 2.5μs | 2.8μs | 5.5μs | 26.4μs | 2.4μs | 396ns | 0.0 | 14.0 | 0B |
| Cancel (head of queue) | 20ns | 40ns | 40ns | 50ns | 50ns | 300ns | 2.0μs | 39ns | 16ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 130ns | 150ns | 150ns | 160ns | 160ns | 170ns | 2.8μs | 148ns | 29ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 20ns | 20ns | 3.1μs | 7ns | 10ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 10ns | 40ns | 50ns | 50ns | 50ns | 380ns | 3.7μs | 43ns | 24ns | 1.0 | 1.0 | 80B |
| Order lookup (hit) | 1ns | 20ns | 30ns | 30ns | 30ns | 210ns | 2.9μs | 23ns | 15ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 50ns | 80ns | 90ns | 100ns | 320ns | 3.5μs | 55ns | 34ns | 0.4 | 0.0 | 13B |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| Default pinned core | Could not pin core 2 |
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 23.5M | 38.0 | 35.0 | 3.3KiB | 645 | 499.6KiB |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


