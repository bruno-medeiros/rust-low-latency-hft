use crate::comparison::MetricDelta;

pub fn fmt_duration(ns: u64) -> String {
    if ns >= 1_000_000 {
        format!("{:.1}ms", ns as f64 / 1_000_000.0)
    } else if ns >= 1_000 {
        format!("{:.1}\u{03bc}s", ns as f64 / 1_000.0)
    } else {
        format!("{ns}ns")
    }
}

pub(crate) fn fmt_duration_f64(ns: f64) -> String {
    fmt_duration(ns as u64)
}

pub(crate) fn fmt_bytes_f64(bytes: f64) -> String {
    if bytes >= 1_048_576.0 {
        format!("{:.1}MiB", bytes / 1_048_576.0)
    } else if bytes >= 1_024.0 {
        format!("{:.1}KiB", bytes / 1_024.0)
    } else {
        format!("{bytes:.0}B")
    }
}

fn fmt_delta(delta: &MetricDelta, value_fn: fn(f64) -> String) -> String {
    let current_str = value_fn(delta.current);
    let pct = delta.pct_change;
    if pct.abs() < 0.5 {
        format!("{current_str} (=)")
    } else {
        let arrow = if pct > 0.0 { "\u{2191}" } else { "\u{2193}" };
        format!("{current_str} ({arrow}{:.1}%)", pct.abs())
    }
}

pub(crate) fn fmt_delta_duration(delta: &MetricDelta) -> String {
    fmt_delta(delta, fmt_duration_f64)
}

pub(crate) fn fmt_delta_bytes(delta: &MetricDelta) -> String {
    fmt_delta(delta, fmt_bytes_f64)
}

pub(crate) fn fmt_delta_count(delta: &MetricDelta) -> String {
    let current_str = format!("{:.1}", delta.current);
    let pct = delta.pct_change;
    if pct.abs() < 0.5 {
        format!("{current_str} (=)")
    } else {
        let arrow = if pct > 0.0 { "\u{2191}" } else { "\u{2193}" };
        format!("{current_str} ({arrow}{:.1}%)", pct.abs())
    }
}

pub(crate) fn fmt_delta_ops_sec(delta: &MetricDelta) -> String {
    let current_str = if delta.current >= 1_000_000.0 {
        format!("{:.1}M", delta.current / 1_000_000.0)
    } else if delta.current >= 1_000.0 {
        format!("{:.1}k", delta.current / 1_000.0)
    } else {
        format!("{:.0}", delta.current)
    };
    let pct = delta.pct_change;
    if pct.abs() < 0.5 {
        format!("{current_str} (=)")
    } else {
        let arrow = if pct > 0.0 { "\u{2191}" } else { "\u{2193}" };
        format!("{current_str} ({arrow}{:.1}%)", pct.abs())
    }
}
