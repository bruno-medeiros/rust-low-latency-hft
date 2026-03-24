mod cli;
mod comparison;
mod format_unit;
mod hardware;
mod renderer;
mod report;
mod runner;
mod runtime_tuning;

pub use cli::CliArgs;
pub use format_unit::fmt_duration;
pub use hardware::HardwareInfo;
pub use renderer::{MarkdownRenderer, Renderer, TextRenderer};
pub use report::{
    AllocStats, BenchReport, BenchReportSection, LatencyScenario, LatencyStats, ReportMetadata,
    ScenarioResult, ThroughputScenario,
};
pub use runner::{BenchRunner, RunMode, core_pinning_disabled_by_env};
pub use runtime_tuning::{RuntimeTuningInfo, append_runtime_tuning_params};

pub use stats_alloc::{self, INSTRUMENTED_SYSTEM, StatsAlloc};
