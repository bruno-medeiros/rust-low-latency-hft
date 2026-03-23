//! Convert bench-tool JSON reports into github-action-benchmark custom JSON
//! (`customSmallerIsBetter` / `customBiggerIsBetter`).
//!
//! Criterion benches are handled separately in CI: run `cargo bench` with
//! `--output-format bencher` and pass that file to the action with `tool: 'cargo'`
//! (same line format as nightly libtest `cargo bench`; see upstream
//! `extractCargoResult` in github-action-benchmark).
//!
//! See <https://github.com/benchmark-action/github-action-benchmark#custom-benchmark-file>

use std::fs;
use std::path::PathBuf;

use bench_tool::BenchReport;
use clap::{ArgAction, Parser};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "github-action-json")]
struct Args {
    /// Title prefix for metric names, then path to a bench-tool JSON report (repeatable).
    #[arg(
        long = "report",
        value_names = ["TITLE", "PATH"],
        num_args = 2,
        action = ArgAction::Append,
        required = true
    )]
    reports: Vec<String>,
    #[arg(long)]
    out_latency: PathBuf,
    #[arg(long)]
    out_throughput: PathBuf,
}

#[derive(Serialize)]
struct Metric {
    name: String,
    unit: &'static str,
    value: f64,
}

fn extract_from_report(report: &BenchReport, title_prefix: &str) -> (Vec<Metric>, Vec<Metric>) {
    let mut latency = Vec::new();
    let mut throughput = Vec::new();

    for section in &report.sections {
        let title = section.title.trim();
        let title_part = if title.is_empty() {
            String::new()
        } else {
            format!("{title} / ")
        };

        for s in &section.latency_scenarios {
            latency.push(Metric {
                name: format!(
                    "{title_prefix} / {title_part}{} (mean latency)",
                    s.name
                ),
                unit: "ns",
                value: s.latency.mean_ns,
            });
        }

        for s in &section.throughput_scenarios {
            throughput.push(Metric {
                name: format!("{title_prefix} / {title_part}{}", s.name),
                unit: "ops/s",
                value: s.throughput_ops_per_sec,
            });
        }
    }

    (latency, throughput)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.reports.len() % 2 != 0 {
        return Err("--report requires TITLE and PATH pairs".into());
    }

    let mut latency_all: Vec<Metric> = Vec::new();
    let mut throughput_all: Vec<Metric> = Vec::new();

    for chunk in args.reports.chunks_exact(2) {
        let title = &chunk[0];
        let path = PathBuf::from(&chunk[1]);
        let report = BenchReport::load_json(&path)?;
        let (lat, thr) = extract_from_report(&report, title);
        latency_all.extend(lat);
        throughput_all.extend(thr);
    }

    fs::write(
        &args.out_latency,
        format!("{}\n", serde_json::to_string_pretty(&latency_all)?),
    )?;
    fs::write(
        &args.out_throughput,
        format!("{}\n", serde_json::to_string_pretty(&throughput_all)?),
    )?;

    Ok(())
}
