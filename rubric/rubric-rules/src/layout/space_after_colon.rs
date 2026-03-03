use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAfterColon;

impl Rule for SpaceAfterColon {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterColon"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
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
                    None if b == b'#' => break,
                    None => {}
                }

                if b == b':' {
                    // Skip `::` (double colon)
                    if j + 1 < len && bytes[j + 1] == b':' {
                        j += 2;
                        continue;
                    }
                    // Skip `:` at end of line
                    if j + 1 >= len {
                        j += 1;
                        continue;
                    }
                    // Skip `:` followed by `]` — POSIX character class closing delimiter
                    // (e.g., `[:word:]`, `[:alpha:]`) or array access. Never a hash key.
                    let next = bytes[j + 1];
                    if next == b']' {
                        j += 1;
                        continue;
                    }
                    // Skip `:` followed by space, newline
                    if next != b' ' && next != b'\n' && next != b'\r' {
                        // Check that the colon is a hash key colon (preceded by a word char)
                        let preceded_by_word = j > 0 && (bytes[j - 1].is_ascii_alphanumeric() || bytes[j - 1] == b'_');
                        if preceded_by_word {
                            let line_start = ctx.line_start_offsets[i];
                            let colon_pos = line_start + j as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Missing space after colon.".into(),
                                range: TextRange::new(colon_pos, colon_pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
                j += 1;
            }
        }

        diags
    }
}
