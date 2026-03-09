use std::alloc::System;
use std::collections::BTreeMap;
use std::hint::black_box;

use core_affinity::CoreId;
use hdrhistogram::Histogram;
use quanta::Clock;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

use crate::report::{AllocStats, BenchReport, LatencyStats, ScenarioResult};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

pub struct BenchRunner {
    title: String,
    warmup_iters: u64,
    sample_iters: u64,
    params: BTreeMap<String, String>,
    pin_core: Option<usize>,
    results: Vec<ScenarioResult>,
    clock: Clock,
}

impl BenchRunner {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            warmup_iters: 10_000,
            sample_iters: 100_000,
            params: BTreeMap::new(),
            pin_core: None,
            results: Vec::new(),
            clock: Clock::new(),
        }
    }

    /// Pin the benchmark thread to a specific CPU core for more consistent latency measurements.
    /// Call this before running scenarios; on unsupported platforms this is a no-op.
    pub fn pin_core(mut self, core: usize) -> Self {
        self.pin_core = Some(core);
        self
    }

    pub fn warmup_iters(mut self, n: u64) -> Self {
        self.warmup_iters = n;
        self
    }

    pub fn sample_iters(mut self, n: u64) -> Self {
        self.sample_iters = n;
        self
    }

    pub fn param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn run<State, S, F>(&mut self, name: &str, setup: S, mut op: F)
    where
        S: Fn() -> State,
        F: FnMut(&mut State),
    {
        let allocator = GLOBAL;
        if let Some(core) = self.pin_core {
            let core_id = CoreId { id: core };
            if !core_affinity::set_for_current(core_id) {
                let reason = match core_affinity::get_core_ids() {
                    Some(cores) => {
                        format!("core {core} not available (available: 0..{})", cores.len())
                    }
                    None => "core affinity not supported on this platform".to_string(),
                };
                eprintln!("\n  warning: CPU pinning failed — {reason}; continuing without pinning");
                self.pin_core = None;
            }
        }

        eprint!("  {name} ... ");

        let mut hist =
            Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).expect("histogram creation");

        let mut total_allocs = 0u64;
        let mut total_deallocs = 0u64;
        let mut total_bytes = 0u64;

        for _ in 0..self.warmup_iters {
            let mut state = setup();
            #[allow(clippy::unit_arg)] // black_box(()) forces op() to be evaluated for benchmarking
            black_box(op(&mut state));
        }

        for _ in 0..self.sample_iters {
            let mut state = setup();

            let region = Region::new(allocator);

            let start = self.clock.raw();
            #[allow(clippy::unit_arg)] // black_box(()) forces op() to be evaluated for benchmarking
            black_box(op(&mut state));
            let end = self.clock.raw();
            let elapsed_ns = self.clock.delta_as_nanos(start, end);

            let stats = region.change();

            hist.record(elapsed_ns.max(1)).expect("histogram record");

            total_allocs += stats.allocations as u64 + stats.reallocations as u64;
            total_deallocs += stats.deallocations as u64;
            total_bytes += stats.bytes_allocated as u64;
        }

        let samples = self.sample_iters;

        self.results.push(ScenarioResult {
            name: name.to_string(),
            samples,
            latency: LatencyStats {
                min_ns: hist.min(),
                p50_ns: hist.value_at_quantile(0.50),
                p90_ns: hist.value_at_quantile(0.90),
                p95_ns: hist.value_at_quantile(0.95),
                p99_ns: hist.value_at_quantile(0.99),
                p999_ns: hist.value_at_quantile(0.999),
                max_ns: hist.max(),
                mean_ns: hist.mean(),
                stdev_ns: hist.stdev(),
            },
            allocations: AllocStats {
                total_allocs,
                total_deallocs,
                total_bytes,
                avg_allocs_per_op: total_allocs as f64 / samples as f64,
                avg_deallocs_per_op: total_deallocs as f64 / samples as f64,
                avg_bytes_per_op: total_bytes as f64 / samples as f64,
            },
        });

        eprintln!("done");
    }

    pub fn finish(self) -> BenchReport {
        BenchReport::build(
            self.title,
            self.warmup_iters,
            self.sample_iters,
            self.params,
            self.pin_core,
            self.results,
        )
    }
}
