use serde::{Deserialize, Serialize};

use crate::report::BenchReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub baseline_title: String,
    pub baseline_timestamp: String,
    pub current_title: String,
    pub current_timestamp: String,
    pub scenarios: Vec<ScenarioComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioComparison {
    pub name: String,
    pub latency: LatencyComparison,
    pub allocations: AllocComparison,
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

impl BenchReport {
    pub fn compare(&self, baseline: &BenchReport) -> Comparison {
        let mut scenarios = Vec::new();

        for current in &self.scenarios {
            if let Some(base) = baseline.scenarios.iter().find(|s| s.name == current.name) {
                let cl = &current.latency;
                let bl = &base.latency;
                let ca = &current.allocations;
                let ba = &base.allocations;

                scenarios.push(ScenarioComparison {
                    name: current.name.clone(),
                    latency: LatencyComparison {
                        min: MetricDelta::from_u64(bl.min_ns, cl.min_ns),
                        p50: MetricDelta::from_u64(bl.p50_ns, cl.p50_ns),
                        p90: MetricDelta::from_u64(bl.p90_ns, cl.p90_ns),
                        p95: MetricDelta::from_u64(bl.p95_ns, cl.p95_ns),
                        p99: MetricDelta::from_u64(bl.p99_ns, cl.p99_ns),
                        p999: MetricDelta::from_u64(bl.p999_ns, cl.p999_ns),
                        max: MetricDelta::from_u64(bl.max_ns, cl.max_ns),
                        mean: MetricDelta::new(bl.mean_ns, cl.mean_ns),
                    },
                    allocations: AllocComparison {
                        avg_allocs_per_op: MetricDelta::new(
                            ba.avg_allocs_per_op,
                            ca.avg_allocs_per_op,
                        ),
                        avg_deallocs_per_op: MetricDelta::new(
                            ba.avg_deallocs_per_op,
                            ca.avg_deallocs_per_op,
                        ),
                        avg_bytes_per_op: MetricDelta::new(
                            ba.avg_bytes_per_op,
                            ca.avg_bytes_per_op,
                        ),
                    },
                });
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
