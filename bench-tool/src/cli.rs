use std::path::PathBuf;

use clap::Parser;

use crate::comparison::Comparison;
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

        let mut text = report.to_text();
        if let Some(cmp) = &comparison {
            text.push_str(&cmp.to_text());
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
            let mut md = report.to_markdown();
            if let Some(cmp) = &comparison {
                md.push_str(&cmp.to_markdown());
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
