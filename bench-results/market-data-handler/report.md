
# market-data-handler: tick-to-trade

| Property | Value |
|----------|-------|
| Timestamp | 2026-04-23T17:19:51Z |
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
| Isolated CPUs | none listed (empty /sys/.../isolated; add isolcpus=… boot param for isolation) |
| Swap | none active (/proc/swaps header only) |
| Turbo / boost | disabled (AMD cpufreq boost=0) |

## Tick-to-trade pipeline

| Property | Value |
|----------|-------|
| T0_definition | quanta::Clock::raw() after recvmmsg() returns |
| T1_definition | quanta::Clock::raw() before OutboundBuf write (excludes sendto syscall) |
| book_events_accepted | 50000 |
| messages_decoded | 50000 |
| messages_sent | 50000 |
| orders_emitted | 49999 |
| packets_received | 50000 |
| pipeline_pin_core | pin core 2 |
| samples_recorded | 49999 |
| sender_pin_core | pin core 3 |

### Latency

| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |
|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|
| Tick-to-trade | 40ns | 2.6μs | 4.7μs | 5.0μs | 9.9μs | 43.1μs | 81.7μs | 2.8μs | 3.1μs | 0.0 | 0.0 | 0B |

