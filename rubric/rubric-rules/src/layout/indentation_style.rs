use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct IndentationStyle;

fn extract_heredoc_terminator_is(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            i += 2;
            if i < n && (line[i] == b'-' || line[i] == b'~') { i += 1; }
            if i < n && (line[i] == b'\'' || line[i] == b'"' || line[i] == b'`') { i += 1; }
            let start = i;
            while i < n && (line[i].is_ascii_alphanumeric() || line[i] == b'_') { i += 1; }
            if i > start { return Some(line[start..i].to_vec()); }
        } else { i += 1; }
    }
    None
}

impl Rule for IndentationStyle {
    fn name(&self) -> &'static str {
        "Layout/IndentationStyle"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<Vec<u8>> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            let bytes = line.as_bytes();

            // ── Heredoc body ─────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator_is(bytes) {
                    in_heredoc = Some(term);
                }
            }

            // Count leading tabs
            let tab_count = bytes.iter().take_while(|&&b| b == b'\t').count();
            if tab_count > 0 {
                let line_start = ctx.line_start_offsets[i] as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use spaces for indentation instead of tabs.".into(),
                    range: TextRange::new(line_start, line_start + tab_count as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        let _ = diag;
        None
    }
}
