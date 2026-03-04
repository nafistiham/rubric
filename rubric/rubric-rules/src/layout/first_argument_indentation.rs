use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FirstArgumentIndentation;

/// Extract the heredoc terminator word from a line containing `<<~TERM`,
/// `<<-TERM`, or `<<TERM` (optionally quoted).  Returns `None` when no
/// heredoc opener is present.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i + 1 < len {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
                i += 1;
                continue;
            }
            Some(_) => {
                i += 1;
                continue;
            }
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
                i += 1;
                continue;
            }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') {
                j += 1;
            }
            if j < len && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`') {
                j += 1;
            }
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

impl Rule for FirstArgumentIndentation {
    fn name(&self) -> &'static str {
        "Layout/FirstArgumentIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track heredoc state: when Some(term), we are inside a heredoc body
        // and skip all lines until we see a line whose trimmed content equals `term`.
        let mut in_heredoc: Option<String> = None;

        for i in 0..n {
            let line = &lines[i];

            // Handle heredoc body/end
            if let Some(ref terminator) = in_heredoc.clone() {
                if line.trim() == terminator.as_str() {
                    in_heredoc = None;
                }
                continue; // skip heredoc body lines (including the terminator line itself)
            }

            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect heredoc openers before scanning for `(`.
            // The opener line itself is still valid Ruby and must be checked,
            // so we set the state here and fall through to the check below.
            if let Some(term) = extract_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: still scan the opener line for `(` below.
            }

            // Detect line ending with `(` (multiline call start)
            if trimmed.ends_with('(') && i + 1 < n {
                let call_indent = line.len() - trimmed.len();
                let expected_indent = call_indent + 2;
                let next_line = &lines[i + 1];
                let actual_indent = next_line.len() - next_line.trim_start().len();
                // Only flag if the next line is not empty and has wrong indentation
                if !next_line.trim().is_empty() && actual_indent != expected_indent {
                    let line_start = ctx.line_start_offsets[i + 1] as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "First argument should be indented {} spaces, not {}.",
                            expected_indent, actual_indent
                        ),
                        range: TextRange::new(line_start, line_start + actual_indent as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
