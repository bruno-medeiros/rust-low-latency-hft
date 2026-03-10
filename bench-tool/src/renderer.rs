use crate::comparison::Comparison;
use crate::format_unit::{
    fmt_bytes_f64, fmt_delta_bytes, fmt_delta_count, fmt_delta_duration, fmt_delta_ops_sec,
    fmt_duration, fmt_duration_f64,
};
use crate::report::{BenchReport, ScenarioResult};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Renderer trait
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub trait Renderer {
    fn render_header(&self, out: &mut String, title: &str, props: &[(&str, String)]);
    fn render_table_start(&self, out: &mut String, title: Option<&str>, headers: &[&str]);
    fn render_table_row(&self, out: &mut String, headers: &[&str], cells: &[String]);

    fn render(&self, report: &BenchReport) -> String {
        let mut out = String::new();
        let m = &report.metadata;

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
        props.push((
            "Samples",
            format!(
                "{} (warmup: {})",
                m.settings.sample_iters, m.settings.warmup_iters
            ),
        ));
        for (k, v) in &m.settings.params {
            props.push((k.as_str(), v.clone()));
        }

        self.render_header(&mut out, &m.title, &props);

        // ── Latency table ──
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

        let latency: Vec<_> = report
            .scenarios
            .iter()
            .filter_map(|s| match s {
                ScenarioResult::Latency(l) => Some(l),
                _ => None,
            })
            .collect();

        self.render_table_start(&mut out, None, latency_headers);
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
            self.render_table_row(&mut out, latency_headers, &cells);
        }

        // ── Throughput table ──
        let throughput: Vec<_> = report
            .scenarios
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
            ];
            self.render_table_start(&mut out, Some("Throughput"), throughput_headers);
            for t in &throughput {
                let cells = vec![
                    t.name.clone(),
                    format!("{:.0}", t.throughput_ops_per_sec),
                    format!("{:.1}", t.allocations.avg_allocs_per_op),
                    format!("{:.1}", t.allocations.avg_deallocs_per_op),
                    fmt_bytes_f64(t.allocations.avg_bytes_per_op),
                ];
                self.render_table_row(&mut out, throughput_headers, &cells);
            }
        }

        out.push('\n');
        out
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Text
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct TextRenderer;

impl TextRenderer {
    fn col_width(i: usize, header: &str) -> usize {
        if i == 0 {
            header.len().max(28)
        } else {
            header.len().max(10)
        }
    }
}

impl Renderer for TextRenderer {
    fn render_header(&self, out: &mut String, title: &str, props: &[(&str, String)]) {
        out.push_str(&format!("\n  {} \u{2014} Latency Report\n\n", title));
        let label_width = props.iter().map(|(k, _)| k.len()).max().unwrap_or(0) + 1;
        for (key, value) in props {
            out.push_str(&format!(
                "  {:<width$} {}\n",
                format!("{key}:"),
                value,
                width = label_width
            ));
        }
    }

    fn render_table_start(&self, out: &mut String, title: Option<&str>, headers: &[&str]) {
        out.push('\n');
        if let Some(t) = title {
            out.push_str(&format!("  {t}\n"));
        }
        out.push_str("  ");
        for (i, h) in headers.iter().enumerate() {
            let w = Self::col_width(i, h);
            let indent = if i == 0 { "" } else { " " };
            out.push_str(&format!("{indent}{:>w$}", h));
        }
        out.push('\n');

        let total: usize = Self::col_width(0, headers[0])
            + headers[1..]
                .iter()
                .enumerate()
                .map(|(j, h)| 1 + Self::col_width(j + 1, h))
                .sum::<usize>();
        out.push_str(&format!("  {}\n", "\u{2500}".repeat(total)));
    }

    fn render_table_row(&self, out: &mut String, headers: &[&str], cells: &[String]) {
        out.push_str("  ");
        for (i, (cell, h)) in cells.iter().zip(headers.iter()).enumerate() {
            let w = Self::col_width(i, h);
            let indent = if i == 0 { "" } else { " " };
            out.push_str(&format!("{indent}{:>w$}", cell));
        }
        out.push('\n');
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Markdown
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_header(&self, out: &mut String, title: &str, props: &[(&str, String)]) {
        out.push_str(&format!("# {}\n\n", title));
        out.push_str("| Property | Value |\n");
        out.push_str("|----------|-------|\n");
        for (key, value) in props {
            out.push_str(&format!("| {} | {} |\n", key, value));
        }
    }

    fn render_table_start(&self, out: &mut String, title: Option<&str>, headers: &[&str]) {
        match title {
            Some(t) => out.push_str(&format!("\n### {}\n\n", t)),
            None => out.push_str("\n## Results\n\n"),
        }
        out.push('|');
        for h in headers {
            out.push_str(&format!(" {} |", h));
        }
        out.push('\n');
        out.push('|');
        for h in headers {
            out.push_str(&format!("{}|", "-".repeat(h.len() + 2)));
        }
        out.push('\n');
    }

    fn render_table_row(&self, out: &mut String, _headers: &[&str], cells: &[String]) {
        out.push('|');
        for cell in cells {
            out.push_str(&format!(" {} |", cell));
        }
        out.push('\n');
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Comparison formatting
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

impl Comparison {
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("failed to serialize comparison")
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "\n  Comparison vs baseline: \"{}\" ({})\n\n",
            self.baseline_title, self.baseline_timestamp
        ));

        use crate::comparison::ScenarioComparison as ScCmp;

        // ── Latency comparison ──
        let latency_scenarios: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| matches!(s, ScCmp::Latency { .. }))
            .collect();
        out.push_str(&format!(
            "  {:<30} {:>18} {:>18} {:>18} {:>18} {:>14} {:>14} {:>14}\n",
            "Operation", "p50", "p99", "p99.9", "mean", "allocs/op", "deallocs/op", "bytes/op"
        ));
        out.push_str(&format!("  {}\n", "\u{2500}".repeat(132)));

        for s in &latency_scenarios {
            let ScCmp::Latency {
                name,
                latency,
                allocations,
            } = s
            else {
                unreachable!()
            };
            out.push_str(&format!(
                "  {:<30} {:>18} {:>18} {:>18} {:>18} {:>14} {:>14} {:>14}\n",
                name,
                fmt_delta_duration(&latency.p50),
                fmt_delta_duration(&latency.p99),
                fmt_delta_duration(&latency.p999),
                fmt_delta_duration(&latency.mean),
                fmt_delta_count(&allocations.avg_allocs_per_op),
                fmt_delta_count(&allocations.avg_deallocs_per_op),
                fmt_delta_bytes(&allocations.avg_bytes_per_op),
            ));
        }

        // ── Throughput comparison ──
        let throughput_scenarios: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| matches!(s, ScCmp::Throughput { .. }))
            .collect();
        if !throughput_scenarios.is_empty() {
            out.push_str(&format!(
                "\n  Throughput\n  {:<30} {:>14} {:>14} {:>14} {:>14}\n",
                "Operation", "ops/sec", "allocs/op", "deallocs/op", "bytes/op"
            ));
            out.push_str(&format!("  {}\n", "\u{2500}".repeat(90)));

            for s in &throughput_scenarios {
                let ScCmp::Throughput {
                    name,
                    throughput_ops_per_sec,
                    allocations,
                } = s
                else {
                    unreachable!()
                };
                out.push_str(&format!(
                    "  {:<30} {:>14} {:>14} {:>14} {:>14}\n",
                    name,
                    fmt_delta_ops_sec(throughput_ops_per_sec),
                    fmt_delta_count(&allocations.avg_allocs_per_op),
                    fmt_delta_count(&allocations.avg_deallocs_per_op),
                    fmt_delta_bytes(&allocations.avg_bytes_per_op),
                ));
            }
        }

        out.push('\n');
        out
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "\n## Comparison vs Baseline\n\n> Baseline: \"{}\" ({})\n\n",
            self.baseline_title, self.baseline_timestamp
        ));

        use crate::comparison::ScenarioComparison as ScCmp;

        let latency_scenarios: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| matches!(s, ScCmp::Latency { .. }))
            .collect();
        out.push_str(
            "| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |\n",
        );
        out.push_str(
            "|-----------|-----|-----|-------|------|-----------|-------------|----------|\n",
        );

        for s in &latency_scenarios {
            let ScCmp::Latency {
                name,
                latency,
                allocations,
            } = s
            else {
                unreachable!()
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                name,
                fmt_delta_duration(&latency.p50),
                fmt_delta_duration(&latency.p99),
                fmt_delta_duration(&latency.p999),
                fmt_delta_duration(&latency.mean),
                fmt_delta_count(&allocations.avg_allocs_per_op),
                fmt_delta_count(&allocations.avg_deallocs_per_op),
                fmt_delta_bytes(&allocations.avg_bytes_per_op),
            ));
        }

        let throughput_scenarios: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| matches!(s, ScCmp::Throughput { .. }))
            .collect();
        if !throughput_scenarios.is_empty() {
            out.push_str("\n### Throughput\n\n");
            out.push_str("| Operation | ops/sec | allocs/op | deallocs/op | bytes/op |\n");
            out.push_str("|-----------|---------|-----------|-------------|----------|\n");

            for s in &throughput_scenarios {
                let ScCmp::Throughput {
                    name,
                    throughput_ops_per_sec,
                    allocations,
                } = s
                else {
                    unreachable!()
                };
                out.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    name,
                    fmt_delta_ops_sec(throughput_ops_per_sec),
                    fmt_delta_count(&allocations.avg_allocs_per_op),
                    fmt_delta_count(&allocations.avg_deallocs_per_op),
                    fmt_delta_bytes(&allocations.avg_bytes_per_op),
                ));
            }
        }

        out.push('\n');
        out
    }
}
