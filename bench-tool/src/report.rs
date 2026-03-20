use std::collections::BTreeMap;
use std::path::Path;

use limit_order_book::CountingEventSink;
use serde::{Deserialize, Serialize};

use chrono::Utc;

use crate::format_unit::{fmt_bytes_f64, fmt_duration_f64};
use crate::hardware::{HardwareInfo, detect_clock_source, detect_rustc_version};
use crate::{Renderer, fmt_duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReport {
    pub metadata: ReportMetadata,
    pub sections: Vec<BenchReportSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReportSection {
    pub title: String,
    pub params: BTreeMap<String, String>,
    pub scenarios: Vec<ScenarioResult>,
}

impl BenchReportSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            params: BTreeMap::new(),
            scenarios: Vec::new(),
        }
    }

    pub fn add_param(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.params.insert(key.into(), value.into());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub title: String,
    pub timestamp: String,
    pub hardware: HardwareInfo,
    pub rustc_version: String,
    /// Clock source used for latency measurements (e.g. "TSC (RDTSC via quanta)").
    #[serde(default)]
    pub clock_source: String,
    /// Note about CPU core pinning (e.g. "Thread pinned to core 2" or "No CPU pinning applied").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_pinning_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyScenario {
    pub name: String,
    pub samples: u64,
    pub latency: LatencyStats,
    pub allocations: AllocStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputScenario {
    pub name: String,
    pub samples: u64,
    pub throughput_ops_per_sec: f64,
    pub allocations: AllocStats,
    /// Total allocations (allocs + reallocs) during setup phase.
    #[serde(default)]
    pub setup_allocs: u64,
    /// Total bytes allocated during setup phase.
    #[serde(default)]
    pub setup_bytes: u64,
    /// Event counts from the run (counting event sink).
    pub event_counts: CountingEventSink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScenarioResult {
    Latency(LatencyScenario),
    Throughput(ThroughputScenario),
}

impl ScenarioResult {
    pub fn name(&self) -> &str {
        match self {
            Self::Latency(s) => &s.name,
            Self::Throughput(s) => &s.name,
        }
    }

    pub fn allocations(&self) -> &AllocStats {
        match self {
            Self::Latency(s) => &s.allocations,
            Self::Throughput(s) => &s.allocations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min_ns: u64,
    pub p50_ns: u64,
    pub p90_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub p999_ns: u64,
    pub max_ns: u64,
    pub mean_ns: f64,
    pub stdev_ns: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocStats {
    pub total_allocs: u64,
    #[serde(default)]
    pub total_deallocs: u64,
    pub total_bytes: u64,
    pub avg_allocs_per_op: f64,
    #[serde(default)]
    pub avg_deallocs_per_op: f64,
    pub avg_bytes_per_op: f64,
}

impl BenchReport {
    /// Creates a report with metadata and empty sections.
    pub fn new_with_metadata(title: String, pin_core: Option<usize>) -> Self {
        let cpu_pinning_note = pin_core.map(|c| format!("Benchmark thread pinned to core {c}"));

        Self {
            metadata: ReportMetadata {
                title,
                timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                hardware: HardwareInfo::detect(),
                rustc_version: detect_rustc_version(),
                clock_source: detect_clock_source(),
                cpu_pinning_note,
            },
            sections: Vec::new(),
        }
    }

    // REVIEW
    pub(crate) fn all_scenarios(&self) -> impl Iterator<Item = &ScenarioResult> {
        self.sections.iter().flat_map(|s| s.scenarios.iter())
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("failed to serialize report")
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn save_json(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_json_pretty())
    }

    pub fn load_json(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&json)?)
    }
}

fn render_section_scenarios<R: Renderer>(
    out: &mut String,
    renderer: &R,
    scenarios: &[ScenarioResult],
) {
    let latency: Vec<_> = scenarios
        .iter()
        .filter_map(|s| match s {
            ScenarioResult::Latency(l) => Some(l),
            _ => None,
        })
        .collect();

    if !latency.is_empty() {
        renderer.render_heading(out, 3, "Latency");
        let latency_headers = &[
            "Operation",
            "min",
            "p50",
            "p90",
            "p95",
            "p99",
            "p99.9",
            "max",
            "mean",
            "stdev",
            "allocs/op",
            "deallocs/op",
            "bytes/op",
        ];
        renderer.render_table_start(out, latency_headers);
        for ls in &latency {
            let cells = vec![
                ls.name.clone(),
                fmt_duration(ls.latency.min_ns),
                fmt_duration(ls.latency.p50_ns),
                fmt_duration(ls.latency.p90_ns),
                fmt_duration(ls.latency.p95_ns),
                fmt_duration(ls.latency.p99_ns),
                fmt_duration(ls.latency.p999_ns),
                fmt_duration(ls.latency.max_ns),
                fmt_duration_f64(ls.latency.mean_ns),
                fmt_duration_f64(ls.latency.stdev_ns),
                format!("{:.1}", ls.allocations.avg_allocs_per_op),
                format!("{:.1}", ls.allocations.avg_deallocs_per_op),
                fmt_bytes_f64(ls.allocations.avg_bytes_per_op),
            ];
            renderer.render_table_row(out, latency_headers, &cells);
        }
    }

    let throughput: Vec<_> = scenarios
        .iter()
        .filter_map(|s| match s {
            ScenarioResult::Throughput(t) => Some(t),
            _ => None,
        })
        .collect();

    if !throughput.is_empty() {
        let throughput_headers = &[
            "Scenario",
            "ops/sec",
            "allocs/op",
            "deallocs/op",
            "bytes/op",
            "setup allocs",
            "setup bytes",
        ];
        renderer.render_heading(out, 3, "Throughput");
        renderer.render_table_start(out, throughput_headers);
        for t in &throughput {
            let cells = vec![
                t.name.clone(),
                format!("{:.0}", t.throughput_ops_per_sec),
                format!("{:.1}", t.allocations.avg_allocs_per_op),
                format!("{:.1}", t.allocations.avg_deallocs_per_op),
                fmt_bytes_f64(t.allocations.avg_bytes_per_op),
                format!("{}", t.setup_allocs),
                fmt_bytes_f64(t.setup_bytes as f64),
            ];
            renderer.render_table_row(out, throughput_headers, &cells);
        }
        for t in &throughput {
            let ec = &t.event_counts;
            let event_headers = &["Event type", "Count"];
            renderer.render_heading(out, 5, &format!("{} — event counts", t.name));
            renderer.render_table_start(out, event_headers);
            for (label, count) in [
                ("Accepted", ec.accepted),
                ("Rejected", ec.rejected),
                ("Fill", ec.fill),
                ("Filled", ec.filled),
                ("Cancelled", ec.cancelled),
            ] {
                renderer.render_table_row(
                    out,
                    event_headers,
                    &[label.to_string(), count.to_string()],
                );
            }
        }
        renderer.render_heading(out, 5, "Throughput flamegraph");
        renderer.render_throughput_extra(out);
    }
}

impl BenchReport {
    pub fn render<R: Renderer>(&self, renderer: &R) -> String {
        let mut out = String::new();
        let m = &self.metadata;

        let mut props: Vec<(&str, String)> = vec![
            ("Timestamp", m.timestamp.clone()),
            ("CPU", m.hardware.cpu_model.clone()),
            ("Cores", m.hardware.cpu_cores.to_string()),
            ("Memory", format!("{:.1} GB", m.hardware.memory_gb)),
            ("OS", format!("{} ({})", m.hardware.os, m.hardware.arch)),
            ("Host", m.hardware.hostname.clone()),
            ("Rust", m.rustc_version.clone()),
            ("Clock", m.clock_source.clone()),
        ];
        if let Some(ref note) = m.cpu_pinning_note {
            props.push(("CPU pinning", note.clone()));
        }

        renderer.render_heading(&mut out, 1, &m.title);
        renderer.render_properties(&mut out, &props);

        for section in &self.sections {
            renderer.render_heading(&mut out, 2, &section.title);
            let mut section_props: Vec<(&str, String)> = Vec::new();
            for (k, v) in &section.params {
                section_props.push((k.as_str(), v.clone()));
            }
            if !section_props.is_empty() {
                renderer.render_properties(&mut out, &section_props);
            }
            render_section_scenarios(&mut out, renderer, &section.scenarios);
        }

        out.push('\n');
        out
    }
}
