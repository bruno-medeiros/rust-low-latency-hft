
# market-data-handler: tick-to-trade

| Property | Value |
|----------|-------|
| Timestamp | 2026-04-29T13:54:59Z |
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
| T0_definition | quanta::Clock::raw() after recvmmsg() returns |
| T1_definition | quanta::Clock::raw() before OutboundBuf write (excludes sendto syscall) |
| Turbo / boost | disabled (AMD cpufreq boost=0) |
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

| Operation | min | p50 | p90 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| In-order packets | 30ns | 40ns | 50ns | 1.6μs | 2.0μs | 7.6μs | 78ns | 272ns | 0.0 | 0.0 | 2B |

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

| Operation | min | p50 | p90 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Out-of-order inbound | 30ns | 50ns | 400ns | 170.9μs | 175.6μs | 180.1μs | 2.4μs | 19.2μs | 0.0 | 0.0 | 2B |

