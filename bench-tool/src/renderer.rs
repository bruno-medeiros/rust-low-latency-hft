// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Renderer trait
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub trait Renderer {
    fn render_heading(&self, out: &mut String, level: u8, title: &str);
    fn render_properties(&self, out: &mut String, props: &[(&str, String)]);
    fn render_table_start(&self, out: &mut String, headers: &[&str]);
    fn render_table_row(&self, out: &mut String, headers: &[&str], cells: &[String]);

    /// Optional extra content after the "Throughput" section heading (e.g. image link).
    /// Default is no-op.
    fn render_throughput_extra(&self, _out: &mut String) {}
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
    fn render_heading(&self, out: &mut String, _level: u8, title: &str) {
        out.push_str(&format!("\n  {title}\n"));
    }

    fn render_properties(&self, out: &mut String, props: &[(&str, String)]) {
        out.push('\n');
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

    fn render_table_start(&self, out: &mut String, headers: &[&str]) {
        out.push('\n');
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

#[derive(Default)]
pub struct MarkdownRenderer {
    /// If set, render a markdown image link after the ### Throughput heading.
    pub flamegraph_path: Option<String>,
}

impl MarkdownRenderer {
    /// Renderer without flamegraph link.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renderer that adds `![Flame graph](path)` after the ### Throughput section.
    pub fn with_flamegraph(path: impl Into<String>) -> Self {
        Self {
            flamegraph_path: Some(path.into()),
        }
    }
}

impl Renderer for MarkdownRenderer {
    fn render_heading(&self, out: &mut String, level: u8, title: &str) {
        let level = level.max(1) as usize;
        out.push('\n');
        out.push_str(&"#".repeat(level));
        out.push(' ');
        out.push_str(title);
        out.push_str("\n\n");
    }

    fn render_properties(&self, out: &mut String, props: &[(&str, String)]) {
        out.push_str("| Property | Value |\n");
        out.push_str("|----------|-------|\n");
        for (key, value) in props {
            out.push_str(&format!("| {} | {} |\n", key, value));
        }
    }

    fn render_table_start(&self, out: &mut String, headers: &[&str]) {
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

    fn render_throughput_extra(&self, out: &mut String) {
        if let Some(ref path) = self.flamegraph_path {
            out.push_str(&format!("![Flame graph]({path})\n\n"));
        }
    }
}
