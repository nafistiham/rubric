use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ExtraSpacing;

/// Returns true if `line` has a bare `=` at byte column `col`
/// (excluding compound operators `!=`, `<=`, `>=`, `==`, `=>`).
fn has_eq_at_col(line: &str, col: usize) -> bool {
    let bytes = line.as_bytes();
    if col >= bytes.len() || bytes[col] != b'=' {
        return false;
    }
    let prev = if col > 0 { bytes[col - 1] } else { 0 };
    let next = if col + 1 < bytes.len() { bytes[col + 1] } else { 0 };
    // Exclude compound operators: !=, <=, >=, ==, =>
    prev != b'!' && prev != b'<' && prev != b'>' && prev != b'='
        && next != b'=' && next != b'>'
}

impl Rule for ExtraSpacing {
    fn name(&self) -> &'static str {
        "Layout/ExtraSpacing"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            // Skip indentation (leading whitespace)
            let indent_len = line.len() - line.trim_start().len();
            let content = &line[indent_len..];

            // Skip pure comment lines
            let content_trimmed = content.trim_start();
            if content_trimmed.starts_with('#') {
                continue;
            }

            // Scan for consecutive spaces outside strings
            let bytes = content.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;
            while j < len {
                let b = bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment
                    None => {}
                }
                // Check for two or more consecutive spaces
                if b == b' ' && j + 1 < len && bytes[j + 1] == b' ' {
                    let span_start = j;
                    while j < len && bytes[j] == b' ' {
                        j += 1;
                    }
                    let span_end = j;

                    // Skip trailing whitespace — all bytes from span_start to end of
                    // content are spaces, meaning these spaces run to EOL. TrailingWhitespace
                    // handles that case; ExtraSpacing must not double-report it.
                    let rest_all_spaces = bytes[span_end..].iter().all(|&b| b == b' ');
                    if rest_all_spaces {
                        j = span_end;
                        continue;
                    }

                    // Skip alignment spacing after comma — e.g. `[10..10,   0..255]`.
                    // Extra spaces immediately following a `,` are valid Ruby column-alignment style.
                    if span_start > 0 && bytes[span_start - 1] == b',' {
                        j = span_end;
                        continue;
                    }

                    // Skip extra spaces immediately before a `#` comment (comment-alignment).
                    if span_end < len && bytes[span_end] == b'#' {
                        j = span_end;
                        continue;
                    }

                    // Skip column-aligned `=` (e.g., vertically-aligned assignments).
                    if span_end < len && bytes[span_end] == b'=' {
                        let after_eq = if span_end + 1 < len { bytes[span_end + 1] } else { 0 };
                        // Only treat as alignment `=`, not compound operators `==`, `=>`
                        if after_eq != b'=' && after_eq != b'>' {
                            let eq_col = indent_len + span_end;
                            let is_aligned = (i > 0 && has_eq_at_col(&lines[i - 1], eq_col))
                                || (i + 1 < lines.len() && has_eq_at_col(&lines[i + 1], eq_col))
                                || (i > 1 && has_eq_at_col(&lines[i - 2], eq_col))
                                || (i + 2 < lines.len() && has_eq_at_col(&lines[i + 2], eq_col));
                            if is_aligned {
                                j = span_end;
                                continue;
                            }
                        }
                    }

                    let line_start = ctx.line_start_offsets[i] as usize;
                    let abs_start = (line_start + indent_len + span_start) as u32;
                    let abs_end = (line_start + indent_len + span_end) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra spacing detected.".into(),
                        range: TextRange::new(abs_start, abs_end),
                        severity: Severity::Warning,
                    });
                    continue;
                }
                j += 1;
            }
        }

        diags
    }
}
