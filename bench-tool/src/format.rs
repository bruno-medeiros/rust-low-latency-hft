use crate::comparison::Comparison;
use crate::format_unit::{
    fmt_bytes_f64, fmt_delta_bytes, fmt_delta_count, fmt_delta_duration, fmt_duration,
    fmt_duration_f64,
};
use crate::report::BenchReport;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Text output
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

impl BenchReport {
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let m = &self.metadata;

        out.push_str(&format!("\n  {} \u{2014} Latency Report\n", m.title));
        out.push_str(&format!("  {}\n\n", m.timestamp));
        out.push_str(&format!("  CPU:      {}\n", m.hardware.cpu_model));
        out.push_str(&format!("  Cores:    {}\n", m.hardware.cpu_cores));
        out.push_str(&format!("  Memory:   {:.1} GB\n", m.hardware.memory_gb));
        out.push_str(&format!(
            "  OS:       {} ({})\n",
            m.hardware.os, m.hardware.arch
        ));
        out.push_str(&format!("  Rust:     {}\n", m.rustc_version));
        out.push_str(&format!("  Clock:    {}\n", m.clock_source));
        if let Some(ref note) = m.cpu_pinning_note {
            out.push_str(&format!("  CPU pinning:  {note}\n"));
        }
        out.push_str(&format!(
            "  Samples:  {} (warmup: {})\n",
            m.settings.sample_iters, m.settings.warmup_iters
        ));
        for (k, v) in &m.settings.params {
            out.push_str(&format!("  {k}: {v}\n"));
        }

        out.push_str(&format!(
            "\n  {:<28} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10}\n",
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
            "bytes/op"
        ));
        out.push_str(&format!("  {}\n", "\u{2500}".repeat(140)));

        for s in &self.scenarios {
            let l = &s.latency;
            let a = &s.allocations;
            out.push_str(&format!(
                "  {:<28} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10} {:>10} {:>10} {:>10} {:>10}\n",
                s.name,
                fmt_duration(l.min_ns),
                fmt_duration(l.p50_ns),
                fmt_duration(l.p90_ns),
                fmt_duration(l.p95_ns),
                fmt_duration(l.p99_ns),
                fmt_duration(l.p999_ns),
                fmt_duration(l.max_ns),
                fmt_duration_f64(l.mean_ns),
                fmt_duration_f64(l.stdev_ns),
                format!("{:.1}", a.avg_allocs_per_op),
                format!("{:.1}", a.avg_deallocs_per_op),
                fmt_bytes_f64(a.avg_bytes_per_op),
            ));
        }

        out.push('\n');
        out
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Markdown output
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

impl BenchReport {
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let m = &self.metadata;

        out.push_str(&format!("# {}\n\n", m.title));

        // ── Metadata ──
        out.push_str("## Metadata\n\n");
        out.push_str("| Property | Value |\n");
        out.push_str("|----------|-------|\n");
        out.push_str(&format!("| Timestamp | {} |\n", m.timestamp));
        out.push_str(&format!("| CPU | {} |\n", m.hardware.cpu_model));
        out.push_str(&format!("| Cores | {} |\n", m.hardware.cpu_cores));
        out.push_str(&format!("| Memory | {:.1} GB |\n", m.hardware.memory_gb));
        out.push_str(&format!(
            "| OS | {} ({}) |\n",
            m.hardware.os, m.hardware.arch
        ));
        out.push_str(&format!("| Host | {} |\n", m.hardware.hostname));
        out.push_str(&format!("| Rust | {} |\n", m.rustc_version));
        out.push_str(&format!("| Clock | {} |\n", m.clock_source));
        if let Some(ref note) = m.cpu_pinning_note {
            out.push_str(&format!("| CPU pinning | {} |\n", note));
        }

        // ── Settings ──
        out.push_str("\n## Settings\n\n");
        out.push_str("| Setting | Value |\n");
        out.push_str("|---------|-------|\n");
        out.push_str(&format!(
            "| Warmup iterations | {} |\n",
            m.settings.warmup_iters
        ));
        out.push_str(&format!(
            "| Sample iterations | {} |\n",
            m.settings.sample_iters
        ));
        for (k, v) in &m.settings.params {
            out.push_str(&format!("| {k} | {v} |\n"));
        }

        // ── Combined table ──
        out.push_str("\n## Results\n\n");
        out.push_str("| Operation | min | p50 | p90 | p95 | p99 | p99.9 | max | mean | stdev | allocs/op | deallocs/op | bytes/op |\n");
        out.push_str("|-----------|-----|-----|-----|-----|-----|-------|-----|------|-------|-----------|-------------|----------|\n");
        for s in &self.scenarios {
            let l = &s.latency;
            let a = &s.allocations;
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.1} | {:.1} | {} |\n",
                s.name,
                fmt_duration(l.min_ns),
                fmt_duration(l.p50_ns),
                fmt_duration(l.p90_ns),
                fmt_duration(l.p95_ns),
                fmt_duration(l.p99_ns),
                fmt_duration(l.p999_ns),
                fmt_duration(l.max_ns),
                fmt_duration_f64(l.mean_ns),
                fmt_duration_f64(l.stdev_ns),
                a.avg_allocs_per_op,
                a.avg_deallocs_per_op,
                fmt_bytes_f64(a.avg_bytes_per_op),
            ));
        }

        out.push('\n');
        out
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

        out.push_str(&format!(
            "  {:<30} {:>18} {:>18} {:>18} {:>18} {:>14} {:>14} {:>14}\n",
            "Operation", "p50", "p99", "p99.9", "mean", "allocs/op", "deallocs/op", "bytes/op"
        ));
        out.push_str(&format!("  {}\n", "\u{2500}".repeat(132)));

        for s in &self.scenarios {
            out.push_str(&format!(
                "  {:<30} {:>18} {:>18} {:>18} {:>18} {:>14} {:>14} {:>14}\n",
                s.name,
                fmt_delta_duration(&s.latency.p50),
                fmt_delta_duration(&s.latency.p99),
                fmt_delta_duration(&s.latency.p999),
                fmt_delta_duration(&s.latency.mean),
                fmt_delta_count(&s.allocations.avg_allocs_per_op),
                fmt_delta_count(&s.allocations.avg_deallocs_per_op),
                fmt_delta_bytes(&s.allocations.avg_bytes_per_op),
            ));
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

        out.push_str(
            "| Operation | p50 | p99 | p99.9 | mean | allocs/op | deallocs/op | bytes/op |\n",
        );
        out.push_str(
            "|-----------|-----|-----|-------|------|-----------|-------------|----------|\n",
        );

        for s in &self.scenarios {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                s.name,
                fmt_delta_duration(&s.latency.p50),
                fmt_delta_duration(&s.latency.p99),
                fmt_delta_duration(&s.latency.p999),
                fmt_delta_duration(&s.latency.mean),
                fmt_delta_count(&s.allocations.avg_allocs_per_op),
                fmt_delta_count(&s.allocations.avg_deallocs_per_op),
                fmt_delta_bytes(&s.allocations.avg_bytes_per_op),
            ));
        }

        out.push('\n');
        out
    }
}
