mod cli;
mod comparison;
mod format_unit;
mod hardware;
mod renderer;
mod report;
mod runtime_tuning;
mod runner;

pub use cli::CliArgs;
pub use format_unit::fmt_duration;
pub use hardware::HardwareInfo;
pub use renderer::{MarkdownRenderer, Renderer, TextRenderer};
pub use report::{
    AllocStats, BenchReport, BenchReportSection, LatencyScenario, LatencyStats, ReportMetadata,
    ScenarioResult, ThroughputScenario,
};
pub use runtime_tuning::{RuntimeTuningInfo, append_runtime_tuning_params};
pub use runner::{BenchRunner, RunMode};

pub use stats_alloc::{self, INSTRUMENTED_SYSTEM, StatsAlloc};
