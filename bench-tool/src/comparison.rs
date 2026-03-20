use crate::format_unit::{fmt_delta_bytes, fmt_delta_count, fmt_delta_duration, fmt_delta_ops_sec};
use crate::renderer::Renderer;
use crate::report::{LatencyScenario, ThroughputScenario};

pub(crate) fn compare_latency_scenarios(
    baseline: &[LatencyScenario],
    current: &[LatencyScenario],
) -> Vec<ScenarioComparison> {
    let mut scenarios = Vec::new();
    for cur in current {
        let Some(base) = baseline.iter().find(|b| b.name == cur.name) else {
            continue;
        };
        push_latency_pair(&mut scenarios, base, cur);
    }
    scenarios
}

pub(crate) fn compare_throughput_scenarios(
    baseline: &[ThroughputScenario],
    current: &[ThroughputScenario],
) -> Vec<ScenarioComparison> {
    let mut scenarios = Vec::new();
    for cur in current {
        let Some(base) = baseline.iter().find(|b| b.name == cur.name) else {
            continue;
        };
        push_throughput_pair(&mut scenarios, base, cur);
    }
    scenarios
}

fn push_latency_pair(
    scenarios: &mut Vec<ScenarioComparison>,
    l_a: &LatencyScenario,
    l_b: &LatencyScenario,
) {
    let ca = &l_b.allocations;
    let ba = &l_a.allocations;

    let alloc_cmp = AllocComparison {
        avg_allocs_per_op: MetricDelta::new(ba.avg_allocs_per_op, ca.avg_allocs_per_op),
        avg_deallocs_per_op: MetricDelta::new(ba.avg_deallocs_per_op, ca.avg_deallocs_per_op),
        avg_bytes_per_op: MetricDelta::new(ba.avg_bytes_per_op, ca.avg_bytes_per_op),
    };

    scenarios.push(ScenarioComparison::Latency {
        name: l_a.name.clone(),
        latency: LatencyComparison {
            p50: MetricDelta::from_u64(l_a.latency.p50_ns, l_b.latency.p50_ns),
            p99: MetricDelta::from_u64(l_a.latency.p99_ns, l_b.latency.p99_ns),
            p999: MetricDelta::from_u64(l_a.latency.p999_ns, l_b.latency.p999_ns),
            mean: MetricDelta::new(l_a.latency.mean_ns, l_b.latency.mean_ns),
        },
        allocations: alloc_cmp,
    });
}

fn push_throughput_pair(
    scenarios: &mut Vec<ScenarioComparison>,
    t_a: &ThroughputScenario,
    t_b: &ThroughputScenario,
) {
    let ca = &t_b.allocations;
    let ba = &t_a.allocations;

    let alloc_cmp = AllocComparison {
        avg_allocs_per_op: MetricDelta::new(ba.avg_allocs_per_op, ca.avg_allocs_per_op),
        avg_deallocs_per_op: MetricDelta::new(ba.avg_deallocs_per_op, ca.avg_deallocs_per_op),
        avg_bytes_per_op: MetricDelta::new(ba.avg_bytes_per_op, ca.avg_bytes_per_op),
    };

    scenarios.push(ScenarioComparison::Throughput {
        name: t_a.name.clone(),
        throughput_ops_per_sec: MetricDelta::new(
            t_a.throughput_ops_per_sec,
            t_b.throughput_ops_per_sec,
        ),
        allocations: alloc_cmp,
        setup_allocs: MetricDelta::from_u64(t_a.setup_allocs, t_b.setup_allocs),
        setup_bytes: MetricDelta::from_u64(t_a.setup_bytes, t_b.setup_bytes),
    });
}

fn render_latency_comparison_rows<R: Renderer>(
    out: &mut String,
    renderer: &R,
    latency_scenarios: &[(&String, &LatencyComparison, &AllocComparison)],
) {
    let latency_headers = &[
        "Operation",
        "p50",
        "p99",
        "p99.9",
        "mean",
        "allocs/op",
        "deallocs/op",
        "bytes/op",
    ];

    renderer.render_table_start(out, latency_headers);
    for (name, latency, allocations) in latency_scenarios {
        let cells = vec![
            (*name).clone(),
            fmt_delta_duration(&latency.p50),
            fmt_delta_duration(&latency.p99),
            fmt_delta_duration(&latency.p999),
            fmt_delta_duration(&latency.mean),
            fmt_delta_count(&allocations.avg_allocs_per_op),
            fmt_delta_count(&allocations.avg_deallocs_per_op),
            fmt_delta_bytes(&allocations.avg_bytes_per_op),
        ];
        renderer.render_table_row(out, latency_headers, &cells);
    }
}

fn render_throughput_comparison_rows<R: Renderer>(
    out: &mut String,
    renderer: &R,
    throughput_scenarios: &[(
        &String,
        &MetricDelta,
        &AllocComparison,
        &MetricDelta,
        &MetricDelta,
    )],
) {
    let throughput_headers = &[
        "Operation",
        "ops/sec",
        "allocs/op",
        "deallocs/op",
        "bytes/op",
        "setup allocs",
        "setup bytes",
    ];
    renderer.render_table_start(out, throughput_headers);
    for (name, throughput_ops_per_sec, allocations, setup_allocs, setup_bytes) in
        throughput_scenarios
    {
        let cells = vec![
            (*name).clone(),
            fmt_delta_ops_sec(throughput_ops_per_sec),
            fmt_delta_count(&allocations.avg_allocs_per_op),
            fmt_delta_count(&allocations.avg_deallocs_per_op),
            fmt_delta_bytes(&allocations.avg_bytes_per_op),
            fmt_delta_count(setup_allocs),
            fmt_delta_bytes(setup_bytes),
        ];
        renderer.render_table_row(out, throughput_headers, &cells);
    }
}

/// Renders only latency delta rows (and optional #### vs baseline) when embedded in the report.
pub(crate) fn render_latency_comparison_embedded<R: Renderer>(
    out: &mut String,
    renderer: &R,
    scenarios: &[ScenarioComparison],
) {
    let latency_scenarios: Vec<_> = scenarios
        .iter()
        .filter_map(|scenario| match scenario {
            ScenarioComparison::Latency {
                name,
                latency,
                allocations,
            } => Some((name, latency, allocations)),
            _ => None,
        })
        .collect();

    if latency_scenarios.is_empty() {
        return;
    }

    renderer.render_heading(out, 4, "vs baseline");
    render_latency_comparison_rows(out, renderer, &latency_scenarios);
}

/// Renders only throughput delta rows (and #### vs baseline) after the main throughput table.
pub(crate) fn render_throughput_comparison_embedded<R: Renderer>(
    out: &mut String,
    renderer: &R,
    scenarios: &[ScenarioComparison],
) {
    let throughput_scenarios: Vec<_> = scenarios
        .iter()
        .filter_map(|scenario| match scenario {
            ScenarioComparison::Throughput {
                name,
                throughput_ops_per_sec,
                allocations,
                setup_allocs,
                setup_bytes,
            } => Some((
                name,
                throughput_ops_per_sec,
                allocations,
                setup_allocs,
                setup_bytes,
            )),
            _ => None,
        })
        .collect();

    if throughput_scenarios.is_empty() {
        return;
    }

    renderer.render_heading(out, 4, "vs baseline");
    render_throughput_comparison_rows(out, renderer, &throughput_scenarios);
}

#[derive(Debug, Clone)]
pub(crate) enum ScenarioComparison {
    Latency {
        name: String,
        latency: LatencyComparison,
        allocations: AllocComparison,
    },
    Throughput {
        name: String,
        throughput_ops_per_sec: MetricDelta,
        allocations: AllocComparison,
        setup_allocs: MetricDelta,
        setup_bytes: MetricDelta,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct LatencyComparison {
    pub p50: MetricDelta,
    pub p99: MetricDelta,
    pub p999: MetricDelta,
    pub mean: MetricDelta,
}

#[derive(Debug, Clone)]
pub(crate) struct AllocComparison {
    pub avg_allocs_per_op: MetricDelta,
    pub avg_deallocs_per_op: MetricDelta,
    pub avg_bytes_per_op: MetricDelta,
}

#[derive(Debug, Clone)]
pub(crate) struct MetricDelta {
    pub current: f64,
    pub pct_change: f64,
}

impl MetricDelta {
    pub fn new(baseline: f64, current: f64) -> Self {
        let pct_change = if baseline.abs() < f64::EPSILON {
            if current.abs() < f64::EPSILON {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            ((current - baseline) / baseline) * 100.0
        };
        Self { current, pct_change }
    }

    pub fn from_u64(baseline: u64, current: u64) -> Self {
        Self::new(baseline as f64, current as f64)
    }
}
