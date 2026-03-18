# lockfree-queue

Lock-free queues: SPSC (single-producer single-consumer) and MPSC (multi-producer single-consumer).

## TODO

- [ ] **SPSC ring buffer** — Single-producer single-consumer ring buffer over a fixed-size heap-allocated array, with cache-line padding on head/tail to eliminate false sharing
- [ ] **Sequence numbers** — Acquire/Release memory ordering for sequence numbers (no mutex, no `std::sync::Mutex`)
- [ ] **MPSC variant** — Multi-producer variant using an atomic sequence claim
- [ ] **Benchmark** — Producer/consumer throughput (ops/sec) and one-way latency (quanta, HDR histogram)
