use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use chrono::Utc;

use crate::hardware::{HardwareInfo, detect_rustc_version};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReport {
    pub metadata: ReportMetadata,
    pub scenarios: Vec<ScenarioResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub title: String,
    pub timestamp: String,
    pub hardware: HardwareInfo,
    pub settings: BenchSettings,
    pub rustc_version: String,
    /// Note about CPU core pinning (e.g. "Thread pinned to core 2" or "No CPU pinning applied").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_pinning_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSettings {
    pub warmup_iters: u64,
    pub sample_iters: u64,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub name: String,
    pub samples: u64,
    pub latency: LatencyStats,
    pub allocations: AllocStats,
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
    pub(crate) fn build(
        title: String,
        warmup_iters: u64,
        sample_iters: u64,
        params: BTreeMap<String, String>,
        pin_core: Option<usize>,
        scenarios: Vec<ScenarioResult>,
    ) -> Self {
        let cpu_pinning_note = pin_core.map(|c| {
            format!("Benchmark thread pinned to core {c}")
        });

        Self {
            metadata: ReportMetadata {
                title,
                timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                hardware: HardwareInfo::detect(),
                settings: BenchSettings {
                    warmup_iters,
                    sample_iters,
                    params,
                },
                rustc_version: detect_rustc_version(),
                cpu_pinning_note,
            },
            scenarios,
        }
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
