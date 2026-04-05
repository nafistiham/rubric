use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Eval;

impl Rule for Eval {
    fn name(&self) -> &'static str {
        "Security/Eval"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                if let Some(delim) = in_string {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        c if c == delim => {
                            in_string = None;
                        }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                match b {
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                    }
                    b'#' => break, // inline comment — stop
                    b'e' => {
                        // Check for bare `eval` at a word boundary
                        if j + 4 <= n && &bytes[j..j + 4] == b"eval" {
                            // Must not be preceded by alphanumeric, `_`, `.`, or `:`
                            // (`.eval` means it's a method call like `binding.eval`;
                            //  `:eval` is a symbol literal, not a call)
                            let preceded_ok = j == 0
                                || !(bytes[j - 1].is_ascii_alphanumeric()
                                    || bytes[j - 1] == b'_'
                                    || bytes[j - 1] == b'.'
                                    || bytes[j - 1] == b':');

                            // Skip `def eval` — method definitions are not eval calls.
                            let is_method_def = j >= 4 && &bytes[j - 4..j] == b"def ";

                            // Must be followed by `(`, ` `, or `"` or `'` — not alphanumeric or `_`
                            let followed_ok = if j + 4 < n {
                                let next = bytes[j + 4];
                                !next.is_ascii_alphanumeric() && next != b'_'
                            } else {
                                true // end of line
                            };

                            if preceded_ok && followed_ok && !is_method_def {
                                let start = (line_start + j) as u32;
                                let end = (line_start + j + 4) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use of `eval` is a security risk.".into(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += 4;
                                continue;
                            }
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
        }

        diags
    }
}
