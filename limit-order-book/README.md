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

## Benchmarks

### Latency distribution (`lob`)

Measures per-operation latency percentiles (min → p99.9) and heap allocations
using [HdrHistogram](https://crates.io/crates/hdrhistogram) and
[stats_alloc](https://crates.io/crates/stats_alloc). Reports hardware metadata
alongside results for reproducibility.

| Scenario | What it measures |
|---|---|
| **Add (passive)** | Limit order that rests — no match |
| **Add (single fill)** | Limit order that matches exactly one resting order |
| **Add (sweep 5 levels, 50 fills)** | Aggressive order that walks 5 price levels; fills 5 × 10 = 50 resting orders |
| **Market (sweep 10 levels, 100 fills)** | Market order consuming 10 levels × 10 orders = 100 fills |
| **Cancel (head of queue)** | Cancel the first order enqueued at a price level — O(1) best case |
| **Cancel (tail of queue)** | Cancel the last order in a 500-deep queue — exposes the O(n) scan in `PriceLevel::remove` |
| **Spread (BBO query)** | Calls `best_bid` + `best_ask` — the fastest read path; covers both sides of the book |
| **Depth (top 5)** | Top-5 levels query; allocates a `Vec` on every call |
| **Order lookup (hit)** | `HashMap::get` by order ID |
| **Realistic mix (per-op)** | Cycles through 40% passive add / 30% cancel / 20% match / 10% BBO query; histogram captures the latency distribution across the mix |

```bash
# Save report as JSON (for future baseline comparison) and Markdown
cargo bench --bench lob -- --save-json bench-results/baseline.json --save-md bench-results/baseline.md
```

```bash
# Full workflow: compare against baseline and save new one
cargo bench --bench lob -- --save-json bench-results/new.json --save-md bench-results/new.md --baseline bench-results/baseline.json
```

### Criterion (`lob_criterion`)

Two groups that go beyond what the latency bench can show on its own:

| Group | What it covers |
|---|---|
| `add_limit_order/sweep_5_levels` | 5-level sweep at book depths 10 / 100 / 1 000 — shows how BTreeMap traversal scales with the number of levels |
| `cancel/queue_position` | Head vs tail cancel in queues of 100 / 500 / 1 000 orders — turns the O(n) `PriceLevel::remove` scan into a visible scaling curve |

```bash
cargo bench --bench lob_criterion
```

