
# Matching pipeline

| Property | Value |
|----------|-------|
| Timestamp | 2026-03-20T20:11:47Z |
| CPU | Apple M4 Pro |
| Cores | 12 |
| Memory | 24.0 GB |
| OS | Darwin 15.7.4 (aarch64) |
| Host | Mac.mynet |
| Rust | rustc 1.91.1 (ed61e7d7e 2025-11-07) |
| Clock | OS clock (platform fallback via quanta) |

## 

| Property | Value |
|----------|-------|
| queue_slots | 4096 |
| sample | LOBSTER_SampleFiles/GOOG_2012-06-21_34200000_57600000_message_1.csv |

### Throughput

| Scenario | ops/sec | allocs/op | deallocs/op | bytes/op | setup allocs | setup bytes |
|----------|---------|-----------|-------------|----------|--------------|-------------|
| Pipeline (Lobster data) | 18421295 | 0.3 | 0.0 | 2B | 11 | 21.8MiB |

| Scenario | Accepted | Rejected | Fill | Filled | Cancelled |
|----------|----------|----------|------|--------|-----------|
| Pipeline (Lobster data) | 961579 | 224768 | 620090 | 633464 | 325533 |

##### Throughput flamegraph

![Flame graph](flamegraph.svg)


