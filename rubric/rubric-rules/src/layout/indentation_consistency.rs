use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IndentationConsistency;

fn extract_heredoc_terminator_ic(line: &[u8]) -> Option<Vec<u8>> {
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

impl Rule for IndentationConsistency {
    fn name(&self) -> &'static str {
        "Layout/IndentationConsistency"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let mut in_heredoc: Option<Vec<u8>> = None;

        for (i, line) in lines.iter().enumerate() {
            let bytes = line.as_bytes();

            // ── Heredoc body ─────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }
            // Detect heredoc opener (body starts on next line)
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator_ic(bytes) {
                    in_heredoc = Some(term);
                }
            }

            // Check if the leading whitespace mixes tabs and spaces
            let mut has_tab = false;
            let mut has_space_after_tab = false;
            let mut j = 0;
            let mut seen_tab = false;

            while j < bytes.len() {
                match bytes[j] {
                    b'\t' => {
                        has_tab = true;
                        if j > 0 && bytes[j - 1] == b' ' {
                            // space before tab
                            has_space_after_tab = true;
                        }
                        seen_tab = true;
                    }
                    b' ' => {
                        if seen_tab {
                            // space after tab in leading whitespace
                            has_space_after_tab = true;
                        }
                    }
                    _ => break,
                }
                j += 1;
            }

            if has_tab && has_space_after_tab {
                let line_start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Mixed tabs and spaces in indentation.".into(),
                    range: TextRange::new(line_start, line_start + j as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
