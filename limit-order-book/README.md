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

## Non-functional Requirements

- **Latency.** All operations target single-digit microsecond latency. Best bid/ask queries must be the fastest path.
- **Determinism.** Identical input sequences must produce identical output sequences — no non-deterministic behavior.
- **No allocation on the hot path.** The critical matching loop must not trigger heap allocations during steady-state operation.
