
# Limit Order Book (v0)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-24T18:20:25Z |
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
| Isolated CPUs | 2-3,10-11 |
| Swap | none active (/proc/swaps header only) |
| Turbo / boost | disabled (AMD cpufreq boost=0) |

## Latency

| Property | Value |
|----------|-------|
| BENCH_ITERS | 100000 |
| Default pinned core | pin core 2 |
| WARMUP_ITERS | 10000 |
| book_levels | 100 |
| orders_per_level | 10 |

### Latency

| Operation | min | p50 | p90 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Add (passive) | 40ns | 50ns | 60ns | 90ns | 110ns | 911ns | 54ns | 10ns | 1.0 | 0.0 | 32B |
| Add (sweep 5 levels, 50 fills) | 1.3μs | 1.4μs | 1.5μs | 1.6μs | 1.8μs | 5.7μs | 1.4μs | 50ns | 0.0 | 6.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 2.7μs | 2.8μs | 2.9μs | 3.1μs | 3.3μs | 8.3μs | 2.9μs | 77ns | 0.0 | 14.0 | 0B |
| Cancel (head of queue) | 30ns | 40ns | 50ns | 70ns | 160ns | 350ns | 41ns | 10ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 150ns | 160ns | 160ns | 170ns | 180ns | 200ns | 159ns | 4ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 20ns | 60ns | 9ns | 2ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 40ns | 50ns | 60ns | 60ns | 310ns | 571ns | 54ns | 13ns | 1.0 | 1.0 | 80B |
| Order lookup (hit) | 10ns | 30ns | 30ns | 40ns | 140ns | 330ns | 26ns | 7ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 10ns | 60ns | 90ns | 110ns | 190ns | 511ns | 62ns | 19ns | 0.4 | 0.0 | 13B |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| Default pinned core | pin core 2 |
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 19.2M | 38.0 | 35.0 | 3.3KiB | 645 | 499.6KiB |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


