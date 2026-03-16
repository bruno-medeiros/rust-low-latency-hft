use std::alloc::System;
use std::collections::BTreeMap;
use std::hint::black_box;

use core_affinity::CoreId;
use hdrhistogram::Histogram;
use quanta::Clock;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

use limit_order_book::CountingEventSink;

use crate::report::{
    AllocStats, BenchReport, LatencyScenario, LatencyStats, ScenarioResult, ThroughputScenario,
};

fn histogram_to_latency_stats(hist: &Histogram<u64>) -> LatencyStats {
    LatencyStats {
        min_ns: hist.min(),
        p50_ns: hist.value_at_percentile(50.0),
        p90_ns: hist.value_at_percentile(90.0),
        p95_ns: hist.value_at_percentile(95.0),
        p99_ns: hist.value_at_percentile(99.0),
        p999_ns: hist.value_at_percentile(99.9),
        max_ns: hist.max(),
        mean_ns: hist.mean(),
        stdev_ns: hist.stdev(),
    }
}

fn build_alloc_stats(
    total_allocs: u64,
    total_deallocs: u64,
    total_bytes: u64,
    samples: u64,
) -> AllocStats {
    AllocStats {
        total_allocs,
        total_deallocs,
        total_bytes,
        avg_allocs_per_op: total_allocs as f64 / samples as f64,
        avg_deallocs_per_op: total_deallocs as f64 / samples as f64,
        avg_bytes_per_op: total_bytes as f64 / samples as f64,
    }
}

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// Whether to measure per-op latency (setup before each op) or sustained throughput (single state).
#[derive(Debug, Clone, Copy)]
pub enum RunMode {
    Latency,
    Throughput,
}

pub struct BenchRunner {
    title: String,
    warmup_iters: u64,
    sample_iters: u64,
    params: BTreeMap<String, String>,
    pin_core: Option<usize>,
    filter: Option<String>,
    results: Vec<ScenarioResult>,
    clock: Clock,
}

impl BenchRunner {
    pub fn new(title: &str) -> Self {
        let params = BTreeMap::new();
        Self {
            title: title.to_string(),
            warmup_iters: 10_000,
            sample_iters: 100_000,
            params,
            pin_core: None,
            filter: None,
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

    pub fn apply_core_pinning(&mut self) {
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

    pub fn filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    /// Run a scenario. `iters` is the number of iterations for both latency and throughput modes.
    pub fn run_latency<State, S, F>(&mut self, name: &str, setup: S, mut op: F, iters: u64)
    where
        S: Fn() -> State,
        F: FnMut(&mut State),
    {
        if let Some(f) = &self.filter
            && !name.to_lowercase().contains(&f.to_lowercase())
        {
            return;
        }

        eprint!("  {name} ... ");

        for _ in 0..self.warmup_iters {
            let mut state = setup();
            #[allow(clippy::unit_arg)]
            black_box(op(&mut state));
        }

        let mut hist =
            Histogram::<u64>::new_with_bounds(1, 1_000_000_000, 3).expect("histogram creation");

        let allocator = GLOBAL;
        let mut total_allocs = 0u64;
        let mut total_deallocs = 0u64;
        let mut total_bytes = 0u64;

        for _ in 0..iters {
            let mut state = setup();

            let region = Region::new(allocator);

            let start = self.clock.raw();
            #[allow(clippy::unit_arg)]
            black_box(op(&mut state));
            let end = self.clock.raw();
            let elapsed_ns = self.clock.delta_as_nanos(start, end);

            let stats = region.change();

            hist.record(elapsed_ns.max(1)).expect("histogram record");

            total_allocs += stats.allocations as u64 + stats.reallocations as u64;
            total_deallocs += stats.deallocations as u64;
            total_bytes += stats.bytes_allocated as u64;
        }

        self.results.push(ScenarioResult::Latency(LatencyScenario {
            name: name.to_string(),
            samples: iters,
            latency: histogram_to_latency_stats(&hist),
            allocations: build_alloc_stats(total_allocs, total_deallocs, total_bytes, iters),
        }));

        eprintln!("done");
    }

    pub fn run_throughput<State, S, F>(
        &mut self,
        name: &str,
        setup: S,
        mut op: F,
        iters: u64,
    ) -> State
    where
        S: Fn() -> State,
        F: FnMut(&mut State, &mut CountingEventSink, &mut u64),
    {
        if let Some(f) = &self.filter
            && !name.to_lowercase().contains(&f.to_lowercase())
        {
            return setup();
        }

        let allocator = GLOBAL;

        let region = Region::new(allocator);
        let mut state = setup();
        let mut sink = CountingEventSink::default();
        let setup_stats = region.change();
        let setup_total_allocs = setup_stats.allocations as u64 + setup_stats.reallocations as u64;
        let setup_total_bytes = setup_stats.bytes_allocated as u64;
        let mut op_count = 0;
        let start = self.clock.raw();
        for _ in 0..iters {
            #[allow(clippy::unit_arg)]
            black_box(op(&mut state, &mut sink, &mut op_count));
        }
        let end = self.clock.raw();
        let total_ns = self.clock.delta_as_nanos(start, end);
        let stats = region.change();

        let total_allocs = stats.allocations as u64 + stats.reallocations as u64;
        let total_deallocs = stats.deallocations as u64;
        let total_bytes = stats.bytes_allocated as u64 - setup_total_bytes;
        let mean_ns = total_ns as f64 / op_count as f64;
        let throughput_ops_per_sec = 1_000_000_000.0 / mean_ns;

        eprint!(
            "  Total bytes: {} alloc: {} dealloc: {}",
            total_bytes, total_allocs, total_deallocs
        );

        self.results
            .push(ScenarioResult::Throughput(ThroughputScenario {
                name: name.to_string(),
                samples: iters,
                throughput_ops_per_sec,
                allocations: build_alloc_stats(total_allocs, total_deallocs, total_bytes, iters),
                setup_allocs: setup_total_allocs,
                setup_bytes: setup_total_bytes,
                event_counts: sink,
            }));

        eprintln!("done");

        state
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
