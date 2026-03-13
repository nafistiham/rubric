use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UriEscapeUnescape;

impl Rule for UriEscapeUnescape {
    fn name(&self) -> &'static str {
        "Lint/UriEscapeUnescape"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;
            let n = bytes.len();

            for pattern in &["URI.escape", "URI.unescape"] {
                let pat_bytes = pattern.as_bytes();
                let pat_len = pat_bytes.len();
                let mut pos = 0;
                while pos + pat_len <= n {
                    if &bytes[pos..pos + pat_len] == pat_bytes {
                        if !in_string_or_comment(bytes, pos) {
                            let start = (line_start + pos) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "`{}` is deprecated; use `URI::DEFAULT_PARSER` methods instead.",
                                    pattern
                                ),
                                range: TextRange::new(start, start + pat_len as u32),
                                severity: Severity::Warning,
                            });
                        }
                        pos += pat_len;
                    } else {
                        pos += 1;
                    }
                }
            }
        }

        diags
    }
}

/// Returns true if `pos` is inside a string literal or inline comment.
fn in_string_or_comment(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos {
        match in_str {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_str = None; }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => { in_str = Some(bytes[i]); }
            None if bytes[i] == b'#' => return true,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}
