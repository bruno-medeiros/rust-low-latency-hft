
# Limit Order Book (v0)

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-24T16:57:40Z |
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
| Add (passive) | 40ns | 50ns | 60ns | 70ns | 100ns | 150ns | 3.7μs | 53ns | 25ns | 1.0 | 0.0 | 32B |
| Add (sweep 5 levels, 50 fills) | 1.4μs | 1.5μs | 1.5μs | 1.6μs | 1.7μs | 5.0μs | 28.1μs | 1.5μs | 305ns | 0.0 | 6.0 | 0B |
| Market (sweep 10 levels, 100 fills) | 2.7μs | 2.9μs | 3.0μs | 3.0μs | 3.3μs | 6.9μs | 30.4μs | 2.9μs | 460ns | 0.0 | 14.0 | 0B |
| Cancel (head of queue) | 30ns | 50ns | 50ns | 60ns | 60ns | 180ns | 20.9μs | 48ns | 68ns | 0.0 | 0.0 | 0B |
| Cancel (tail of queue) | 150ns | 160ns | 170ns | 170ns | 180ns | 180ns | 4.0μs | 164ns | 36ns | 0.0 | 0.0 | 0B |
| Spread (BBO query) | 1ns | 10ns | 10ns | 10ns | 20ns | 20ns | 160ns | 9ns | 2ns | 0.0 | 0.0 | 0B |
| Depth (top 5) | 30ns | 60ns | 70ns | 80ns | 80ns | 460ns | 23.0μs | 64ns | 82ns | 1.0 | 1.0 | 80B |
| Order lookup (hit) | 1ns | 20ns | 20ns | 30ns | 30ns | 130ns | 350ns | 18ns | 7ns | 0.0 | 0.0 | 0B |
| Realistic mix (per-op) | 1ns | 60ns | 90ns | 100ns | 110ns | 240ns | 3.9μs | 62ns | 33ns | 0.4 | 0.0 | 13B |

## Throughput (realistic mix)

| Property | Value |
|----------|-------|
| Default pinned core | Could not pin core 2 |
| book_levels | 100 |
| orders_per_level | 10 |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Throughput (realistic mix) | 25.8M | 38.0 | 35.0 | 3.3KiB | 645 | 499.6KiB |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Throughput (realistic mix) | 116.0M | 0 | 32.0M | 40.0M | 76.0M |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


