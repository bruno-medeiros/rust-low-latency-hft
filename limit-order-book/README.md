# Limit Order Book (LOB)

A high-performance limit order book engine written in Rust, designed as a learning and portfolio project for HFT (High-Frequency Trading) and low-latency systems programming.

---

## What Is a Limit Order Book?

A limit order book is the core data structure at the heart of every modern electronic exchange (stocks, futures, crypto, FX). It is a real-time, ordered record of all outstanding buy and sell **limit orders** for a single trading instrument (e.g. AAPL, BTC-USD).

### Key Concepts

- **Limit Order**: An instruction to buy or sell a specific quantity at a specific price (or better). It rests in the book until it is filled, cancelled, or expires.
- **Market Order**: An instruction to buy or sell immediately at the best available price. It doesn't rest in the book — it matches against existing limit orders.
- **Bid Side**: All outstanding buy orders, sorted from highest price (best bid) to lowest.
- **Ask Side**: All outstanding sell orders, sorted from lowest price (best ask) to highest.
- **Spread**: The difference between the best ask and the best bid. A tighter spread means a more liquid market.
- **Price Level**: A single price point on one side of the book. A price level aggregates all orders at that price into a queue, typically FIFO (first-in, first-out).
- **Depth**: The number of distinct price levels visible in the book, or the total quantity available across those levels.
- **Top of Book (BBO)**: The Best Bid and Offer — the highest bid and lowest ask. This is the most latency-critical query.

### How It Works

1. A **new limit order** arrives. The matching engine checks if it can immediately match against resting orders on the opposite side (e.g., a buy at $100.05 when the best ask is $100.03).
2. If it **crosses the spread** (price is equal to or better than the opposite best), it matches — partially or fully — generating one or more **trades** (fills).
3. Any **remaining quantity** after matching rests in the book at its limit price. If this is a brand-new order, it joins the back of the FIFO queue at that price level. If it is a resting order that was only partially filled (i.e., an incoming order matched against it but didn't consume its full quantity), it **keeps its position** in the queue — it does not move to the back.
4. A **cancel** request removes a specific resting order by its ID.
5. A **modify/replace** request changes the price and/or quantity of a resting order. Typically, changing the price loses FIFO priority (treated as cancel + new order), while reducing quantity preserves priority.

### Why This Matters for HFT / Low-Latency

The order book is on the **critical path** of every trade. In production HFT systems:

- Matching engines process millions of messages per second.
- Single-operation latency targets are in the **hundreds of nanoseconds to low microseconds**.
- Deterministic, bounded latency matters more than average throughput. Tail latency (p99, p99.9) is what kills you.
- Memory allocation on the hot path is forbidden — everything is pre-allocated or pooled.
- Cache locality dominates performance; data structures are designed around CPU cache lines (64 bytes).

This project lets you confront all of these constraints in a focused, self-contained system.

---

## Functional Requirements

### Order Types

- **Limit Order** — Accept a limit order with: side (buy/sell), price, quantity, and a unique order ID. The order rests in the book if it cannot be immediately filled.

- **Market Order** — Accept a market order with: side and quantity. It matches against the opposite side at the best available prices until fully filled or the book is exhausted. No resting quantity remains.

- **Cancel Order** — Accept a cancel request by order ID. Remove the order from the book. Return success or failure (e.g., order not found, already filled).

- **Modify Order (stretch)** — Accept a modify request by order ID with new price and/or quantity. If price changes, the order loses time priority. If only quantity decreases, priority is preserved.

### Matching Engine

- **Price-Time Priority (FIFO)** — Orders at the same price level are matched in the order they were received. Better-priced orders always match first.

- **Partial Fills** — An incoming order may match against multiple resting orders across one or more price levels. Each match generates a separate fill/trade event.

- **Self-Trade Prevention (stretch)** — If the same participant is on both sides, prevent the trade (cancel the resting order, cancel the incoming order, or cancel both — configurable).

### Book Queries

- **Best Bid / Best Ask (BBO)** — Return the current best bid price and quantity, and best ask price and quantity. This must be the fastest query.

- **Book Depth Snapshot** — Return the top N price levels on each side, with aggregate quantity per level.

- **Order Status** — Given an order ID, return its current state: open (with remaining qty), filled, cancelled, or unknown.

- **Volume at Price** — Return total resting quantity at a specific price level on a given side.

### Trade / Execution Output

- **Trade Events** — Every match produces a trade event containing: aggressor order ID, passive order ID, price, quantity, and timestamp.

- **Order Lifecycle Events** — Emit events for: order accepted, order partially filled, order fully filled, order cancelled, order rejected.

- **Event Sequence Numbering** — Every event gets a monotonically increasing sequence number for deterministic replay.

### Multi-Symbol Support (stretch)

- **Instrument Registry** — Support multiple independent order books, one per instrument/symbol.

- **Symbol Routing** — Incoming messages specify a symbol; the engine routes to the correct book.

### Market Data Feed (stretch)

- **L2 Market Data** — Publish a top-of-book or depth-of-book update after every state change (order add, cancel, trade).

- **Trade Tape** — Publish a stream of all executed trades with price, quantity, aggressor side, and timestamp.

---

## Non-Functional Requirements

### Latency

- **Add order (no match)** — Inserting a non-crossing limit order into the book. Target: < 1 μs p50, < 2 μs p99.

- **Add order (with match)** — Inserting an order that crosses the spread and generates fills. Target: < 2 μs p50, < 5 μs p99.

- **Cancel order** — Removing an order by ID from the book. Target: < 500 ns p50, < 1 μs p99.

- **BBO query** — Returning best bid/ask must be essentially free (O(1), cached). Target: < 100 ns.

- **Deterministic latency** — p99.9 should be no more than 5x the p50. No GC pauses, no unbounded allocations. Minimal variance between runs.

### Throughput

- **Message throughput** — On a single core, for a mix of add/cancel/query operations. Target: > 5 million ops/sec.

- **Burst handling** — The engine must not degrade under burst loads (no dynamic allocation, no lock contention). Target: sustain 10x average load.

### Memory

- **Zero allocation on hot path** — After initial setup, no heap allocation during order processing. Use pre-allocated pools, arena allocators, or fixed-capacity collections.

- **Memory budget** — The book for a single instrument with 100K resting orders should fit in < 50 MB.

- **Cache-friendly layout** — Orders and price levels should be stored in contiguous memory. Avoid pointer-chasing (linked lists, `HashMap` with heap-allocated entries). Prefer arrays, `Vec`-backed structures, or arena-indexed slots.

- **Pre-allocated object pools** — Order objects and price level nodes should come from a pre-sized pool. Reuse slots on cancel/fill rather than deallocating.

### Correctness

- **Deterministic replay** — Given the same sequence of input messages, the engine must produce the exact same sequence of output events. No randomness, no time-dependent branching on the hot path.

- **Price-time priority invariant** — At no point should a later order at the same price level be matched before an earlier one.

- **No lost or phantom orders** — Every accepted order must be trackable and must eventually be filled, cancelled, or explicitly expired. No order should silently disappear.

- **Comprehensive test suite** — Unit tests for every operation. Property-based tests (e.g., with `proptest`) for invariants: book is always sorted, total quantity is conserved, FIFO is maintained.

### Observability and Benchmarking

- **Built-in benchmarks** — Use `criterion` or a custom harness to measure per-operation latency distributions (p50, p95, p99, p99.9, max).

- **Latency histogram** — Record operation latencies in an HdrHistogram for reporting.

- **Statistics endpoint** — Expose: total orders processed, total trades, current book depth, uptime, and latency percentiles.

- **CPU profiling hooks** — Support `perf`, `flamegraph`, or `dhat` profiling with minimal instrumentation overhead. Build with `#[inline]` and LTO in release mode.

### Build and Tooling

- **No `unsafe` initially** — Write the first version entirely in safe Rust. Only introduce `unsafe` in a later optimization pass, with clear justification and safety comments.

- **Release profile tuning** — `Cargo.toml` release profile: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.

- **No external runtime** — No async runtime (tokio, etc.) on the hot path. The matching engine is a synchronous, single-threaded loop. Async is only for I/O at the edges (network, logging).

- **Minimal dependencies** — Keep the core matching engine dependency-free. External crates only for benchmarking, testing, serialization, and network I/O.

### Architecture

- **Single-threaded core** — The matching engine runs on a single pinned CPU core. No locks, no atomics, no shared-memory concurrency on the hot path.

- **Message-driven interface** — The engine consumes a stream of input messages (add, cancel, modify) and produces a stream of output events (accepted, fill, cancelled). Clean separation of I/O and logic.

- **Layered design** — Separate the project into layers: (1) core data structures (order, price level, book), (2) matching engine logic, (3) message codec / serialization, (4) network transport, (5) benchmarks and tests.

- **Pluggable transport (stretch)** — Support both an in-process API (for benchmarks) and a network interface (TCP or UDP) for external clients.

---

## Suggested Data Structure Sketch

This section describes the conceptual data model — not an implementation.

- **Order**: ID, side, price, original quantity, remaining quantity, timestamp (or sequence number).
- **Price Level**: Price, total quantity, a FIFO queue of order references (not heap pointers — arena indices or IDs).
- **Half-Book** (one side): A sorted collection of price levels. Needs efficient insert, remove, and access to the best level. Candidates: `BTreeMap`, sorted `Vec`, or a custom skip list / array-indexed tree.
- **Order Book**: Two half-books (bid and ask) plus an index for O(1) order lookup by ID (for cancels).
- **Object Pool**: A pre-allocated slab of order slots, reused via a free list.

---

## Suggested Implementation Phases

### Phase 1 — Core Data Structures
Build the `Order`, `PriceLevel`, and `OrderBook` types. Implement add and cancel. Write unit tests. No performance tuning yet — get correctness first.

### Phase 2 — Matching Engine
Implement price-time priority matching for limit and market orders. Emit trade events. Add property-based tests for invariants.

### Phase 3 — Benchmarking Baseline
Add `criterion` benchmarks. Measure add, cancel, and match latencies. Profile with `flamegraph`. Identify bottlenecks.

### Phase 4 — Performance Optimization
Replace `BTreeMap` / `HashMap` with cache-friendly structures. Introduce object pooling (arena allocator or slab). Eliminate all allocations on the hot path. Re-benchmark and compare.

### Phase 5 — Message Interface
Define a binary message format (FIX-inspired or custom). Build a synchronous message loop that reads from a buffer, dispatches to the engine, and writes output events.

### Phase 6 — Network Transport (stretch)
Add a TCP or UDP server. Benchmark end-to-end latency (network in -> match -> network out). Explore kernel bypass (io_uring, DPDK) if you want to go deeper.

### Phase 7 — Advanced Features (stretch)
Multi-symbol support. Market data feed. Order modify. Self-trade prevention. Iceberg / hidden orders.

---

## Key Rust Techniques You Will Practice

- Ownership and borrowing for zero-copy message handling.
- Arena allocation and index-based references instead of `Box` / `Rc` / heap pointers.
- `#[repr(C)]` and manual struct layout for cache-line alignment.
- Generics and trait-based abstraction for pluggable components without virtual dispatch (monomorphization).
- `criterion` for statistically rigorous micro-benchmarks.
- `proptest` for property-based testing of invariants.
- `perf` / `flamegraph` / `dhat` for profiling and identifying cache misses.
- Unsafe Rust (later) for SIMD, raw pointers in the arena, or lock-free queues at I/O boundaries.

---

## References and Further Reading

- [How to Build an Exchange](https://jane-street.com/tech-talks/building-an-exchange/) — Jane Street tech talk.
- [Limit Order Book (Wikipedia)](https://en.wikipedia.org/wiki/Order_book_(trading)) — foundational concepts.
- [The Trading and Exchanges Book (Larry Harris)](https://global.oup.com/academic/product/trading-and-exchanges-9780195144703) — the definitive reference on market microstructure.
- [WK Selph — Building a Trading System](https://web.archive.org/web/20110219163448/http://howtohft.wordpress.com/2011/02/15/how-to-build-a-fast-limit-order-book/) — classic blog post on LOB data structures.
- [Rust Performance Book](https://nnethercote.github.io/perf-book/) — practical Rust optimization guide.
