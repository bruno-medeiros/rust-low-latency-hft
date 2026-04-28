
# market-data-handler: tick-to-trade

| Property | Value |
|----------|-------|
| Timestamp | 2026-04-27T16:49:26Z |
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
| messages_sent | 50000 |
| packets_received | 50000 |
| messages_decoded | 50000 |
| reorder_ahead_arrivals | 0 |
| orders_emitted | 49999 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| In-order packets | 40ns | 50ns | 70ns | 80ns | 1.5μs | 2.3μs | 8.3μs | 91ns | 254ns | 2.0 | 3.0 | 177B |

## Tick-to-trade pipeline (out of order inbound)

| Property | Value |
|----------|-------|
| messages_sent | 50000 |
| packets_received | 50000 |
| messages_decoded | 50000 |
| reorder_ahead_arrivals | 20000 |
| orders_emitted | 49999 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Out-of-order inbound | 40ns | 80ns | 290ns | 1.2μs | 141.3μs | 145.7μs | 151.0μs | 2.0μs | 15.9μs | 1.7 | 2.6 | 139B |

