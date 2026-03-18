# Limit Order Book

A single-instrument limit order book written in Rust.

## Operations

### Commands

| Operation | Description |
|---|---|
| **Add limit order** | Insert an order with side, price, quantity, and ID. Immediately matches against the opposite side if prices cross; any remaining quantity rests in the book. |
| **Add market order** | Match against the opposite side at the best available prices until filled. Unfilled remainder is cancelled (never rests). |
| **Cancel order** | Remove a resting order by ID. Rejects if the order is unknown. |

All commands return a sequence of events: `Accepted`, `Fill`, `Filled`, `Cancelled`, or `Rejected`.

### Queries

| Query | Description |
|---|---|
| **Best bid / best ask** | Price and aggregate quantity at the top of each side. |
| **Spread** | Distance between best ask and best bid. |
| **Depth** | Top N price levels on a given side with aggregate quantity per level. |
| **Order lookup** | Current state of a resting order by ID. |

### Matching Rules

- **Price-time (FIFO) priority.** Better prices match first; ties broken by arrival order.
- **Partial fills.** An incoming order can match across multiple resting orders and price levels.
- Every event carries a monotonically increasing sequence number for deterministic replay.

### Non-functional Requirements

- **Latency.** All operations target single-digit microsecond latency. Best bid/ask queries must be the fastest path.
- **Determinism.** Identical input sequences must produce identical output sequences — no non-deterministic behavior.
- **No allocation on the hot path.** The critical matching loop must not trigger heap allocations during steady-state operation.

## Implementation (v1)

The low-latency implementation uses these data structures:

- **Price levels** — a pre-allocated flat array indexed by price tick. Best bid/ask indices are maintained on every insert/remove, making BBO queries O(1) with no traversal.
- **Order queue** — an intrusive doubly-linked list per price level. Orders at the same price are chained through `prev`/`next` pointers stored inside each order slot, giving O(1) insert (append to tail) and O(1) cancel (unlink by pointer) with no shifting or searching.
- **Order storage** — a [slab](https://crates.io/crates/slab) (arena with free-list) holds all live order slots, with a `HashMap<OrderId, SlabKey>` for external-ID lookup. 

All three structures are pre-allocated at startup. The matching hot path performs only array/slab indexing and pointer chasing — no heap allocation, no tree rebalancing, no hash table resizing.

## Benchmarks

### Latency distribution and Throughput (sustained mix)

Run benchmark suite with: limit order book v0 (baseline) and v1, produce report with comparison:

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
cargo bench --bench lob_criterion
```

