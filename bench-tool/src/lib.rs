mod cli;
mod comparison;
mod format;
mod format_unit;
mod hardware;
mod report;
mod runner;

pub use cli::CliArgs;
pub use comparison::{
    AllocComparison, Comparison, LatencyComparison, MetricDelta, ScenarioComparison,
};
pub use format_unit::fmt_duration;
pub use hardware::HardwareInfo;
pub use report::{
    AllocStats, BenchReport, BenchSettings, LatencyStats, ReportMetadata, ScenarioResult,
};
pub use runner::BenchRunner;

pub use stats_alloc::{self, StatsAlloc, INSTRUMENTED_SYSTEM};
