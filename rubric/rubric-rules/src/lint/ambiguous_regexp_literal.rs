use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AmbiguousRegexpLiteral;

fn extract_heredoc_term_arl(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            let mut j = i + 2;
            if j < n && (line[j] == b'-' || line[j] == b'~') { j += 1; }
            if j < n && (line[j] == b'\'' || line[j] == b'"' || line[j] == b'`') { j += 1; }
            let start = j;
            while j < n && (line[j].is_ascii_alphanumeric() || line[j] == b'_') { j += 1; }
            if j > start { return Some(line[start..j].to_vec()); }
        }
        i += 1;
    }
    None
}

impl Rule for AmbiguousRegexpLiteral {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousRegexpLiteral"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<Vec<u8>> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }
            if let Some(term) = extract_heredoc_term_arl(line.as_bytes()) {
                in_heredoc = Some(term);
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect method call followed by space then `/` (regexp without parens)
            // Patterns like `p /` or `puts /` or `print /`
            //
            // Skip when the identifier is a control-flow keyword — in those
            // positions the `/` is unambiguously a regex literal:
            //   `when /pat/`  `if /pat/`  `unless /pat/`  `return /pat/`
            const UNAMBIGUOUS: &[&str] = &[
                "when", "if", "elsif", "unless", "while", "until",
                "return", "yield", "raise", "fail",
                "and", "or", "not", "rescue", "else",
                "def",  // `def /(other)` — method named `/`
            ];

            let bytes = trimmed.as_bytes();
            let n = bytes.len();
            let mut pos = 0;

            // Skip leading identifier (method name)
            while pos < n && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }

            // Check if followed by space then `/`
            if pos > 0 && pos < n && bytes[pos] == b' ' {
                let word = std::str::from_utf8(&bytes[..pos]).unwrap_or("");
                if !UNAMBIGUOUS.contains(&word) {
                    let mut j = pos + 1;
                    while j < n && bytes[j] == b' ' { j += 1; }
                    // Must be `/` (regex start) but NOT `/=` (compound division-assign)
                    // and NOT `/ ` (space after `/` = arithmetic division — unambiguous).
                    // A regex literal `/pattern/` has non-space content after the `/`.
                    if j < n && bytes[j] == b'/'
                        && (j + 1 >= n || (bytes[j + 1] != b'=' && bytes[j + 1] != b' '))
                    {
                        let indent = line.len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let flag_pos = (line_start + indent + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Ambiguous regexp literal; wrap in parentheses to clarify.".into(),
                            range: TextRange::new(flag_pos, flag_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}
