# Limit Order Book

A single-instrument limit order book written in Rust.
It supports both direct matching commands and feed-driven resting-order lifecycle updates (add/reduce/cancel) through its public API.

## Operations

Command and query API docs live on the `LimitOrderBook` trait:
[`src/book_api.rs`](src/book_api.rs).

### Matching Rules

- **Price-time (FIFO) priority.** Better prices match first; ties broken by arrival order.
- **Partial fills.** An incoming order can match across multiple resting orders and price levels.
- Every event carries a monotonically increasing sequence number for deterministic replay.

### Implementation (v0)

A non-optimized, non-low-latency version used as baseline for benchmarks.

### Non-functional Requirements

- **Latency.** All operations target single-digit microsecond latency. Best bid/ask queries must be the fastest path.
- **Determinism.** Identical input sequences must produce identical output sequences — no non-deterministic behavior.
- **No allocation on the hot path.** The critical matching loop must not trigger heap allocations during steady-state operation.

### Implementation (v1)

The low-latency implementation uses these data structures:

- **Price levels** — a pre-allocated flat array indexed by price tick. Best bid/ask indices are maintained on every insert/remove, making BBO queries O(1) with no traversal.
- **Order queue** — an intrusive doubly-linked list per price level. Orders at the same price are chained through `prev`/`next` pointers stored inside each order slot, giving O(1) insert (append to tail) and O(1) cancel (unlink by pointer) with no shifting or searching.
- **Order storage** — a [slab](https://crates.io/crates/slab) (arena with free-list) holds all live order slots, with a `HashMap<OrderId, SlabKey>` for external-ID lookup. 

All three structures are pre-allocated at startup. The matching hot path performs only array/slab indexing and pointer chasing — no heap allocation, no tree rebalancing, no hash table resizing.

## Benchmarks

### Latency distribution and Throughput (sustained mix)

Run the workspace benchmark suite from the **repository root**:

```bash
./run-benchmarks-and-report.sh
```

Measures per-operation latency percentiles (min → p99.9) and heap allocations
using [HdrHistogram](https://crates.io/crates/hdrhistogram) and
[stats_alloc](https://crates.io/crates/stats_alloc). Reports hardware metadata
alongside results for reproducibility. Also adds a flamegraph to throughput scenario using
 [cargo-flamegraph](https://github.com/killercup/cargo-flamegraph)



### Criterion (`lob_criterion`)


```bash
cargo bench -p limit-order-book --bench lob_criterion
```

