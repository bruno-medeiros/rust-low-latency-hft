# Benchmark Methodology

Note: I only have access to an Apple Silicon / ARM machine. So for benchmarks, timing tuning is focused on Mac, even if it's not as precise as x86_64/Linux.

## Timing methodology

The benchmark tooling uses the [`quanta`](https://docs.rs/quanta) crate for high-resolution, low-overhead timing, following standard HFT industry practices where measurement overhead must be negligible.

**Clock source by architecture:**

- **x86_64 (Intel/AMD)**: Uses the CPU's Time Stamp Counter (TSC) via `RDTSC`/`RDTSCP` instructions, achieving ~1-5ns measurement overhead per sample — an order of magnitude lower than `std::time::Instant` (~20-25ns per call via `clock_gettime`).
- **aarch64 (Apple Silicon / ARM)**: Falls back to the OS-provided monotonic clock, which is the best available source on ARM platforms.

## Additional measures for accurate results

- **Core pinning** — benchmark threads are pinned to a specific CPU core to eliminate cross-core migration noise and ensure consistent TSC reads.
- **HDR Histogram** — latency samples are recorded in a high dynamic range histogram (3 significant digits) for accurate tail-latency percentiles (p99, p99.9).
- **Allocation tracking** — heap allocations are tracked per-operation to verify zero-allocation hot paths.
- **Warm-up phase** — a configurable warm-up period ensures instruction and data caches are hot before measurement begins.
- **`black_box`** — prevents the compiler from optimizing away the measured operations.

## macOS tuning for benchmarks

macOS does not support CPU core pinning. However, there are OS-level settings that reduce background noise and improve measurement consistency.

**Before benchmarking:**

```bash
# Disable Spotlight indexing (causes background I/O and CPU spikes)
sudo mdutil -a -i off

# Disable Timer Coalescing (macOS batches timers to save power, adding jitter)
sudo sysctl -w kern.timer.coalescing_enabled=0

# Prevent sleep and Power Nap during benchmark runs
sudo pmset -a disablesleep 1
sudo pmset -a powernap 0
```

Close all non-essential applications (browsers, Slack, Docker, etc.) to minimize contention for CPU time and memory bandwidth.

**After benchmarking — restore normal settings:**

```bash
# Re-enable Spotlight indexing
sudo mdutil -a -i on

# Re-enable Timer Coalescing
sudo sysctl -w kern.timer.coalescing_enabled=1

# Re-enable sleep and Power Nap
sudo pmset -a disablesleep 0
sudo pmset -a powernap 1
```
