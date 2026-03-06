# Low Latency / HFT demos

A repository to showcase low-latency / HFT projects, with benchmarks to demonstrate the performance results. The benchmarks include various latency percentiles, allocations counts, test metadata (hardware and OS config).

## Benchmarks

### Timing methodology

The benchmark tooling uses the [`quanta`](https://docs.rs/quanta) crate for high-resolution, low-overhead timing, following standard HFT industry practices where measurement overhead must be negligible.

**Clock source by architecture:**

- **x86_64 (Intel/AMD)**: Uses the CPU's Time Stamp Counter (TSC) via `RDTSC`/`RDTSCP` instructions, achieving ~1-5ns measurement overhead per sample — an order of magnitude lower than `std::time::Instant` (~20-25ns per call via `clock_gettime`).
- **aarch64 (Apple Silicon / ARM)**: Falls back to the OS-provided monotonic clock, which is the best available source on ARM platforms.

### Additional measures for accurate results

- **Core pinning** — benchmark threads are pinned to a specific CPU core to eliminate cross-core migration noise and ensure consistent TSC reads.
- **HDR Histogram** — latency samples are recorded in a high dynamic range histogram (3 significant digits) for accurate tail-latency percentiles (p99, p99.9).
- **Allocation tracking** — heap allocations are tracked per-operation to verify zero-allocation hot paths.
- **Warm-up phase** — a configurable warm-up period ensures instruction and data caches are hot before measurement begins.
- **`black_box`** — prevents the compiler from optimizing away the measured operations.