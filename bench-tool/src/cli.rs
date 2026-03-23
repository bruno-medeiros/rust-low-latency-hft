use std::path::PathBuf;

use clap::Parser;

use crate::renderer::{MarkdownRenderer, TextRenderer};
use crate::report::BenchReport;

#[derive(Debug, Parser)]
#[command(about = "HFT / low-latency benchmark report tool")]
pub struct CliArgs {
    /// Path to baseline JSON report for comparison
    #[arg(long)]
    pub baseline: Option<PathBuf>,

    /// Save report as JSON to file
    #[arg(long)]
    pub save_json: Option<PathBuf>,

    /// Save report as Markdown to file
    #[arg(long)]
    pub save_md: Option<PathBuf>,

    /// In Markdown output, add an image link after ### Throughput (path from separate command).
    /// Omit PATH to use "flamegraph.svg".
    #[arg(long, value_name = "PATH", num_args = 0..=1, default_missing_value = "flamegraph.svg")]
    pub flamegraph: Option<String>,

    /// Run only scenarios whose name contains this string (case-insensitive)
    #[arg(long)]
    pub filter: Option<String>,

    /// Pin the benchmark thread to this CPU core index (e.g. 2 for an isolated core).
    #[arg(long, value_name = "CORE")]
    pub pin_core: Option<usize>,

    /// LOB implementation version (e.g. v0, v1)
    #[arg(long, default_value = "v1")]
    pub lob_version: String,

    /// Injected by `cargo bench`; accepted and ignored.
    #[arg(long, hide = true)]
    pub bench: bool,
}

impl CliArgs {
    pub fn parse_args() -> Self {
        <Self as clap::Parser>::parse()
    }

    pub fn execute(&self, report: &BenchReport) -> Result<(), Box<dyn std::error::Error>> {
        let baseline_report = match &self.baseline {
            Some(path) => {
                let baseline = BenchReport::load_json(path)?;
                Some(baseline)
            }
            None => None,
        };

        let text = report.render_with_baseline(&TextRenderer, baseline_report.as_ref());
        print!("{text}");

        if let Some(path) = &self.save_json {
            let json = report.to_json_pretty();
            std::fs::write(path, json)?;
            eprintln!("JSON saved to {}", path.display());
        }

        if let Some(path) = &self.save_md {
            let renderer = match &self.flamegraph {
                None => MarkdownRenderer::new(),
                Some(p) => MarkdownRenderer::with_flamegraph(p.clone()),
            };
            let md = report.render_with_baseline(&renderer, baseline_report.as_ref());
            std::fs::write(path, md)?;
            eprintln!("Markdown saved to {}", path.display());
        }

        Ok(())
    }
}
