
# market-data-handler: tick-to-trade

| Property | Value |
|----------|-------|
| Timestamp | 2026-04-28T17:18:58Z |
| CPU | AMD Ryzen 7 7800X3D 8-Core Processor |
| Cores | 16 |
| Memory | 30.5 GB |
| OS | Linux Mint 22.3 (x86_64) |
| Host | bruno-linux |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | TSC (RDTSC via quanta) |
| ASLR | enabled full (randomize_va_space=2) |
| CPU governor | powersave (all 16 CPUs) |
| IRQ affinity (sample) | mixed (64 sampled IRQs; first=0-15) |
| Isolated CPUs | 2-3,10-11 |
| Swap | /swapfile type=file used=2097148 |
| T0_definition | quanta::Clock::raw() after recvmmsg() returns |
| T1_definition | quanta::Clock::raw() before OutboundBuf write (excludes sendto syscall) |
| Turbo / boost | enabled (AMD cpufreq boost=1) |
| bench_sender_pin_core | pin core 3 |
| pipeline_pin_core | pin core 2 |

## Tick-to-trade pipeline (in-order)

| Property | Value |
|----------|-------|
| packets_received | 50000 |
| messages_decoded | 50000 |
| reorder_ahead_arrivals | 0 |
| orders_emitted | 49999 |
| total_allocs | 1 |
| total_deallocs | 4 |
| total_bytes | 94720 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| In-order packets | 80ns | 90ns | 110ns | 120ns | 1.5μs | 2.7μs | 27.7μs | 136ns | 293ns | 0.0 | 0.0 | 2B |

## Tick-to-trade pipeline (out of order inbound)

| Property | Value |
|----------|-------|
| packets_received | 50000 |
| messages_decoded | 50000 |
| reorder_ahead_arrivals | 20000 |
| orders_emitted | 49999 |
| total_allocs | 1 |
| total_deallocs | 4 |
| total_bytes | 94720 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Out-of-order inbound | 80ns | 110ns | 430ns | 691ns | 138.9μs | 143.4μs | 220.7μs | 2.0μs | 15.7μs | 0.0 | 0.0 | 2B |

