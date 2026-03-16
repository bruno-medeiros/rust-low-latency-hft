use serde::{Deserialize, Serialize};

use crate::format_unit::{fmt_delta_bytes, fmt_delta_count, fmt_delta_duration, fmt_delta_ops_sec};
use crate::renderer::{MarkdownRenderer, Renderer, TextRenderer};
use crate::report::{BenchReport, ScenarioResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub baseline_title: String,
    pub baseline_timestamp: String,
    pub current_title: String,
    pub current_timestamp: String,
    pub scenarios: Vec<ScenarioComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScenarioComparison {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyComparison {
    pub min: MetricDelta,
    pub p50: MetricDelta,
    pub p90: MetricDelta,
    pub p95: MetricDelta,
    pub p99: MetricDelta,
    pub p999: MetricDelta,
    pub max: MetricDelta,
    pub mean: MetricDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocComparison {
    pub avg_allocs_per_op: MetricDelta,
    #[serde(default)]
    pub avg_deallocs_per_op: MetricDelta,
    pub avg_bytes_per_op: MetricDelta,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricDelta {
    pub baseline: f64,
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
        Self {
            baseline,
            current,
            pct_change,
        }
    }

    pub fn from_u64(baseline: u64, current: u64) -> Self {
        Self::new(baseline as f64, current as f64)
    }
}

impl Comparison {
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("failed to serialize comparison")
    }

    pub fn render<R: Renderer>(&self, renderer: &R) -> String {
        let mut out = String::new();

        renderer.render_heading(&mut out, 2, "Comparison vs Baseline");
        renderer.render_properties(
            &mut out,
            &[(
                "Baseline",
                format!("\"{}\" ({})", self.baseline_title, self.baseline_timestamp),
            )],
        );

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
        let latency_scenarios: Vec<_> = self
            .scenarios
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

        renderer.render_table_start(&mut out, latency_headers);
        for (name, latency, allocations) in latency_scenarios {
            let cells = vec![
                name.clone(),
                fmt_delta_duration(&latency.p50),
                fmt_delta_duration(&latency.p99),
                fmt_delta_duration(&latency.p999),
                fmt_delta_duration(&latency.mean),
                fmt_delta_count(&allocations.avg_allocs_per_op),
                fmt_delta_count(&allocations.avg_deallocs_per_op),
                fmt_delta_bytes(&allocations.avg_bytes_per_op),
            ];
            renderer.render_table_row(&mut out, latency_headers, &cells);
        }

        let throughput_headers = &[
            "Operation",
            "ops/sec",
            "allocs/op",
            "deallocs/op",
            "bytes/op",
            "setup allocs",
            "setup bytes",
        ];
        let throughput_scenarios: Vec<_> = self
            .scenarios
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

        if !throughput_scenarios.is_empty() {
            renderer.render_heading(&mut out, 3, "Throughput");
            renderer.render_table_start(&mut out, throughput_headers);
            for (name, throughput_ops_per_sec, allocations, setup_allocs, setup_bytes) in
                throughput_scenarios
            {
                let cells = vec![
                    name.clone(),
                    fmt_delta_ops_sec(throughput_ops_per_sec),
                    fmt_delta_count(&allocations.avg_allocs_per_op),
                    fmt_delta_count(&allocations.avg_deallocs_per_op),
                    fmt_delta_bytes(&allocations.avg_bytes_per_op),
                    fmt_delta_count(setup_allocs),
                    fmt_delta_bytes(setup_bytes),
                ];
                renderer.render_table_row(&mut out, throughput_headers, &cells);
            }
        }

        out.push('\n');
        out
    }

    pub fn to_text(&self) -> String {
        self.render(&TextRenderer)
    }

    pub fn to_markdown(&self) -> String {
        self.render(&MarkdownRenderer::new())
    }
}

impl BenchReport {
    pub fn compare(&self, baseline: &BenchReport) -> Comparison {
        let mut scenarios = Vec::new();

        for current in &self.scenarios {
            let base = match baseline
                .scenarios
                .iter()
                .find(|s| s.name() == current.name())
            {
                Some(b) => b,
                None => continue,
            };

            let ca = current.allocations();
            let ba = base.allocations();

            let alloc_cmp = AllocComparison {
                avg_allocs_per_op: MetricDelta::new(ba.avg_allocs_per_op, ca.avg_allocs_per_op),
                avg_deallocs_per_op: MetricDelta::new(
                    ba.avg_deallocs_per_op,
                    ca.avg_deallocs_per_op,
                ),
                avg_bytes_per_op: MetricDelta::new(ba.avg_bytes_per_op, ca.avg_bytes_per_op),
            };

            match (base, current) {
                (ScenarioResult::Latency(l_a), ScenarioResult::Latency(l_b)) => {
                    scenarios.push(ScenarioComparison::Latency {
                        name: l_a.name.clone(),
                        latency: LatencyComparison {
                            min: MetricDelta::from_u64(l_a.latency.min_ns, l_b.latency.min_ns),
                            p50: MetricDelta::from_u64(l_a.latency.p50_ns, l_b.latency.p50_ns),
                            p90: MetricDelta::from_u64(l_a.latency.p90_ns, l_b.latency.p90_ns),
                            p95: MetricDelta::from_u64(l_a.latency.p95_ns, l_b.latency.p95_ns),
                            p99: MetricDelta::from_u64(l_a.latency.p99_ns, l_b.latency.p99_ns),
                            p999: MetricDelta::from_u64(l_a.latency.p999_ns, l_b.latency.p999_ns),
                            max: MetricDelta::from_u64(l_a.latency.max_ns, l_b.latency.max_ns),
                            mean: MetricDelta::new(l_a.latency.mean_ns, l_b.latency.mean_ns),
                        },
                        allocations: alloc_cmp,
                    });
                }
                (ScenarioResult::Throughput(t_a), ScenarioResult::Throughput(t_b)) => {
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
                _ => {}
            }
        }

        Comparison {
            baseline_title: baseline.metadata.title.clone(),
            baseline_timestamp: baseline.metadata.timestamp.clone(),
            current_title: self.metadata.title.clone(),
            current_timestamp: self.metadata.timestamp.clone(),
            scenarios,
        }
    }
}
