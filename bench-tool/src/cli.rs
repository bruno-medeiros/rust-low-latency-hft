use std::path::PathBuf;

use clap::Parser;

use crate::comparison::Comparison;
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
        let comparison = match &self.baseline {
            Some(path) => {
                let baseline = BenchReport::load_json(path)?;
                Some(report.compare(&baseline))
            }
            None => None,
        };

        let mut text = report.render(&TextRenderer);
        if let Some(cmp) = &comparison {
            text.push_str(&cmp.render(&TextRenderer));
        }
        print!("{text}");

        if let Some(path) = &self.save_json {
            let json = match &comparison {
                Some(cmp) => serde_json::to_string_pretty(&CombinedReport {
                    report,
                    comparison: cmp,
                })?,
                None => report.to_json_pretty(),
            };
            std::fs::write(path, json)?;
            eprintln!("JSON saved to {}", path.display());
        }

        if let Some(path) = &self.save_md {
            let renderer = match &self.flamegraph {
                None => MarkdownRenderer::new(),
                Some(p) => MarkdownRenderer::with_flamegraph(p.clone()),
            };
            let mut md = report.render(&renderer);
            if let Some(cmp) = &comparison {
                md.push_str(&cmp.render(&renderer));
            }
            std::fs::write(path, md)?;
            eprintln!("Markdown saved to {}", path.display());
        }

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct CombinedReport<'a> {
    report: &'a BenchReport,
    comparison: &'a Comparison,
}
