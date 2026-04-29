use std::alloc::System;
use std::hint::black_box;

use crate::BenchReportSection;
use crate::report::{
    AllocStats, BenchReport, LatencyScenario, LatencyStats, ScenarioResult, ThroughputScenario,
};
use core_affinity::CoreId;
use hdrhistogram::Histogram;
use limit_order_book::CountingEventSink;
use quanta::Clock;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

/// When this environment variable is set (any value), core pinning is disabled.
const BENCH_TOOL_DISABLE_CORE_PINNING: &str = "BENCH_TOOL_DISABLE_CORE_PINNING";

#[inline]
pub fn core_pinning_disabled_by_env() -> bool {
    std::env::var_os(BENCH_TOOL_DISABLE_CORE_PINNING).is_some()
}

fn histogram_to_latency_stats(hist: &Histogram<u64>) -> LatencyStats {
    LatencyStats {
        min_ns: hist.min(),
        p50_ns: hist.value_at_percentile(50.0),
        p90_ns: hist.value_at_percentile(90.0),
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
    let samples = samples.max(1);
    AllocStats {
        total_allocs,
        total_deallocs,
        total_bytes,
        avg_allocs_per_op: total_allocs as f64 / samples as f64,
        avg_deallocs_per_op: total_deallocs as f64 / samples as f64,
        avg_bytes_per_op: total_bytes as f64 / samples as f64,
    }
}

pub fn alloc_stats_from_basic_stats(stats: stats_alloc::Stats, samples: u64) -> AllocStats {
    let total_allocs = stats.allocations as u64 + stats.reallocations as u64;
    let total_deallocs = stats.deallocations as u64;
    let total_bytes = stats.bytes_allocated as u64;
    build_alloc_stats(total_allocs, total_deallocs, total_bytes, samples)
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

    filter: Option<String>,
    results: Vec<ScenarioResult>,
    clock: Clock,
}

impl BenchRunner {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            warmup_iters: 10_000,
            sample_iters: 100_000,
            filter: None,
            results: Vec::new(),
            clock: Clock::new(),
        }
    }
    /// Pin the **current** thread to `pin_core` for latency/throughput stability.
    /// Returns a short status string for report sections. On failure, prints a warning and returns the reason.
    pub fn pin_to_isolated_core(&self, pin_core: u32) -> String {
        if core_pinning_disabled_by_env() {
            return format!(
                "Core pinning disabled ({})",
                BENCH_TOOL_DISABLE_CORE_PINNING
            );
        }
        let core_id = CoreId {
            id: pin_core as usize,
        };
        if core_affinity::set_for_current(core_id) {
            format!("pin core {}", pin_core)
        } else {
            let reason = format!("Could not pin core {pin_core}");
            eprintln!("warning: {reason}. Continuing without pinning.");
            reason
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

    pub fn filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    pub fn initial_report(&self) -> BenchReport {
        BenchReport::new_with_metadata(self.title.clone())
    }

    pub fn push_section(&mut self, mut section: BenchReportSection, report: &mut BenchReport) {
        for r in self.results.drain(..) {
            match r {
                ScenarioResult::Latency(l) => section.latency_scenarios.push(l),
                ScenarioResult::Throughput(t) => section.throughput_scenarios.push(t),
            }
        }
        report.sections.push(section);
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

    pub fn run_throughput<State, S, FSink, F>(
        &mut self,
        name: &str,
        setup: S,
        sink_op: FSink,
        mut op: F,
        iters: u64,
    ) where
        S: Fn() -> State,
        FSink: Fn(State) -> CountingEventSink,
        F: FnMut(&mut State, &mut u64),
    {
        if let Some(f) = &self.filter
            && !name.to_lowercase().contains(&f.to_lowercase())
        {
            return;
        }

        let allocator = GLOBAL;

        let region = Region::new(allocator);
        let mut state = setup();
        let setup_stats = region.change();
        let setup_total_allocs = setup_stats.allocations as u64 + setup_stats.reallocations as u64;
        let setup_total_bytes = setup_stats.bytes_allocated as u64;
        let mut op_count = 0;
        let start = self.clock.raw();
        for _ in 0..iters {
            #[allow(clippy::unit_arg)]
            black_box(op(&mut state, &mut op_count));
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

        let sink = sink_op(state);

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
    }
}
