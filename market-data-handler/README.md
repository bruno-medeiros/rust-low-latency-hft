# market-data-handler

Full UDP market data feed handler: receive raw datagrams, decode ITCH-style messages,
maintain a limit order book, run a minimal strategy stub, and measure **tick-to-trade**
latency end-to-end.

## Pipeline

```
UdpReceiver 
    │
    ▼
MoldUDP64-lite decode (T0 timestamp after decode_packet)
    │
    ▼
ReorderBuffer
    │
    ▼
ItchDecoder
    │
    ▼
ItchToBookAdapter → LimitOrderBook
    │
    ▼
QuoterState (strategy stub)
    │
    ▼
OutboundBuf (order encode)
    │
    ▼
LatencyRecorder (record T1 timestamp) 
```

All of the above runs **on a single pinned thread**. There is no cross-thread queue on
the hot path — book updates and strategy decisions are inline, as in production
feed handlers. A side-channel (SPSC journal, fan-out) can be added off-path without
touching the latency-critical loop.

## Tick-to-trade definition

| Point | Definition |
|---|---|
| **T0** | `quanta::Clock::raw()` immediately after `recvmmsg()` returns (before `decode_packet()` and reorder processing). |
| **T1** | `quanta::Clock::raw()` immediately before writing to `OutboundBuf` — order bytes encoded, ready for `sendto` |
| **Latency** | `clock.delta_as_nanos(T0, T1)` — excludes kernel TX path |

Measured on loopback UDP (single host). The kernel RX path latency is visible in T0 but
the kernel TX path is excluded from the measurement. Strategy is intentionally trivial
(see below) so the number isolates pipeline latency rather than alpha complexity.


## Benchmark

```bash
# Apply OS tuning, run the tick-to-trade bench, revert
sudo ./run-benchmarks-linux.sh

# Or run directly (without tuning)
cargo bench --bench tick_to_trade -p market-data-handler
```

The benchmark:

1. Pre-encodes 50,000 synthetic ITCH `AddOrder` messages as MoldUDP64-lite UDP packets
   (no alloc in the hot loop).
2. Sender thread (pinned to core A) sends all packets to a loopback socket.
3. Pipeline (pinned to core B) receives via `recvmmsg`, processes inline, records T0→T1
   into an HDR histogram (feed is in-order on loopback).
4. Reports percentile table via `bench-tool` in the same format as the other crates.

