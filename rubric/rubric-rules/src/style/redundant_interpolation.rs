use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantInterpolation;

/// Returns true if the token starting at `start` in `bytes` is a double-quoted
/// string whose only content is a single `#{...}` interpolation — i.e., the
/// string matches exactly `"#{...}"` with nothing else inside.
///
/// Returns `Some(end_index)` (exclusive, pointing past the closing `"`) on
/// match, `None` otherwise.
fn is_pure_interpolation(bytes: &[u8], start: usize) -> Option<usize> {
    let n = bytes.len();

    // Must start with `"#{ `
    if start + 3 >= n {
        return None;
    }
    if bytes[start] != b'"' || bytes[start + 1] != b'#' || bytes[start + 2] != b'{' {
        return None;
    }

    // Walk forward from the `{`, tracking brace depth, until we find the
    // matching `}`.  After the `}` the very next byte must be `"` and there
    // must be no other content between the opening `"` and the closing `"`.
    let mut depth: usize = 1;
    let mut i = start + 3; // first byte after `#{`

    while i < n {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    // The character immediately after `}` must be the closing `"`
                    let close_brace = i;
                    let after = close_brace + 1;
                    if after < n && bytes[after] == b'"' {
                        return Some(after + 1);
                    }
                    // There is extra content between `}` and the closing `"` — not pure
                    return None;
                }
            }
            b'\\' => {
                // skip escaped character inside interpolation
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    None
}

impl Rule for RedundantInterpolation {
    fn name(&self) -> &'static str {
        "Style/RedundantInterpolation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip full-line comments
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[line_idx] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut i = 0usize;
            let mut in_single_quote = false;

            while i < n {
                let b = bytes[i];

                // Track single-quoted strings (no interpolation inside)
                if in_single_quote {
                    match b {
                        b'\\' => i += 1, // skip escaped char
                        b'\'' => in_single_quote = false,
                        _ => {}
                    }
                    i += 1;
                    continue;
                }

                match b {
                    b'#' => break, // inline comment — stop scanning
                    b'\'' => {
                        in_single_quote = true;
                        i += 1;
                    }
                    b'"' => {
                        if let Some(end) = is_pure_interpolation(bytes, i) {
                            let start_offset = (line_start + i) as u32;
                            let end_offset = (line_start + end) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use `to_s` instead of interpolation.".into(),
                                range: TextRange::new(start_offset, end_offset),
                                severity: Severity::Warning,
                            });
                            i = end;
                        } else {
                            // Skip past this double-quoted string without flagging
                            i += 1;
                            while i < n {
                                match bytes[i] {
                                    b'\\' => i += 1,
                                    b'"' => {
                                        i += 1;
                                        break;
                                    }
                                    _ => {}
                                }
                                i += 1;
                            }
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
        }

        diags
    }
}
