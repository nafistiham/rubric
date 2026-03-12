use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLines;

fn extract_heredoc_terminator_el(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            if j < len && matches!(bytes[j], b'\'' | b'"' | b'`') { j += 1; }
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

impl Rule for EmptyLines {
    fn name(&self) -> &'static str {
        "Layout/EmptyLines"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut blank_run = 0usize;
        let mut in_heredoc: Option<String> = None;
        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines (including blank lines inside heredocs)
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                blank_run = 0; // reset so lines after heredoc don't count
                continue;
            }
            // Detect heredoc opener
            if let Some(term) = extract_heredoc_terminator_el(line) {
                in_heredoc = Some(term);
                // Fall through: opener line itself is real Ruby
            }

            if line.trim().is_empty() {
                blank_run += 1;
                if blank_run >= 2 {
                    let start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra blank line detected.".into(),
                        range: TextRange::new(start, start),
                        severity: Severity::Warning,
                    });
                }
            } else {
                blank_run = 0;
            }
        }
        diags
    }
}
