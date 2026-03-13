use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub const DEFAULT_MAX: usize = 120;

pub struct LineLength {
    pub max: usize,
}

impl Default for LineLength {
    fn default() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

/// Extract the heredoc terminator from a heredoc opener line.
/// Supports <<~TERM, <<-TERM, <<TERM with optional quotes.
fn heredoc_terminator(line: &str) -> Option<String> {
    let marker = if let Some(p) = line.find("<<~") {
        Some(&line[p + 3..])
    } else if let Some(p) = line.find("<<-") {
        Some(&line[p + 3..])
    } else if let Some(p) = line.find("<<") {
        let after = &line[p + 2..];
        if after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '"' || c == '\'') {
            Some(after)
        } else {
            None
        }
    } else {
        None
    }?;
    let rest = marker.trim_start_matches(|c: char| c == '\'' || c == '"');
    let term: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if term.is_empty() { None } else { Some(term) }
}

/// Return true when the line (or its comment/string) contains an http/https URI.
/// Rubocop's AllowURI: true skips any line that contains a URI that would push
/// the length over the limit.
fn contains_uri(line: &str) -> bool {
    line.contains("http://") || line.contains("https://")
}

impl Rule for LineLength {
    fn name(&self) -> &'static str {
        "Layout/LineLength"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let max = self.max;

        // Heredoc tracking — skip lines inside heredoc bodies (AllowHeredoc parity).
        let mut in_heredoc = false;
        let mut heredoc_terminator_str = String::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // ── Heredoc body ─────────────────────────────────────────────────
            if in_heredoc {
                if line.trim() == heredoc_terminator_str.as_str() {
                    in_heredoc = false;
                    heredoc_terminator_str.clear();
                }
                // Skip the line regardless (heredoc content or terminator).
                continue;
            }

            // ── Unicode-aware character count (not byte count) ────────────────
            // Rubocop counts Unicode codepoints; Rust's str::len() counts bytes.
            let char_len = line.chars().count();

            if char_len > max {
                // AllowURI parity: skip lines that contain an http/https URI.
                if !contains_uri(line) {
                    let start = ctx.line_start_offsets[i];
                    let end = start + line.len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Line is {char_len} characters (max is {max})."),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                }
            }

            // ── Enter heredoc mode for subsequent lines ───────────────────────
            if let Some(term) = heredoc_terminator(line) {
                in_heredoc = true;
                heredoc_terminator_str = term;
            }
        }
        diags
    }
}
