# market-data-handler

Full UDP market data feed handler: receive raw datagrams, decode ITCH-style messages,
maintain a limit order book, run a minimal strategy stub, and measure **tick-to-trade**
latency end-to-end.

## Pipeline

```
recvmmsg (batched UDP RX)
    │
    ▼
MoldUDP64-lite decode          ← frame.rs: zero-copy slice iterator
    │
    ▼
ReorderRing (gap detection)    ← reorder.rs: seq-numbered ring, in-order drain
    │
    ▼
ItchDecoder                    ← decode.rs: zero-copy, borrows from recv buffer
    │
    ▼
FeedBookAdapter → LimitOrderBook ← feed_book.rs + limit-order-book crate
    │
    ▼
QuoterState (strategy stub)    ← strategy.rs: top-of-book cross-spread quoter
    │
    ▼
OutboundBuf (order encode)     ← outbound.rs: [u8; 18] stack buffer, zero alloc
    │
    ▼
T1 timestamp → LatencyRecorder ← latency.rs: TSC via quanta + hdrhistogram
```

All of the above runs **on a single pinned thread**. There is no cross-thread queue on
the hot path — book updates and strategy decisions are inline, as in production
feed handlers. A side-channel (SPSC journal, fan-out) can be added off-path without
touching the latency-critical loop.

## Tick-to-trade definition

| Point | Definition |
|---|---|
| **T0** | `quanta::Clock::raw()` immediately after `recvmmsg(2)` returns — first byte of the batch is available in userspace |
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
   into an HDR histogram.
4. Reports percentile table via `bench-tool` in the same format as the other crates.

### Sample results (untuned, Ryzen 7 7800X3D, powersave governor)

| min | p50 | p90 | p99 | p99.9 | max |
|---|---|---|---|---|---|
| 70 ns | 3.5 μs | 6.2 μs | 12.0 μs | 52.6 μs | 87.4 μs |

> Numbers above are from an untuned run (powersave CPU governor, no isolated cores, no
> hugepages). With `isolcpus`, performance governor, and disabled turbo boost the p50 and
> p99 drop significantly. See [Benchmark Methodology](../Benchmark-Methodology.md) and
> `run-benchmarks-linux.sh` for the full tuning procedure.
