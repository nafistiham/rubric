use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DoubleNegation;

impl Rule for DoubleNegation {
    fn name(&self) -> &'static str {
        "Style/DoubleNegation"
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
                    b'!' => {
                        if j + 1 < n && bytes[j + 1] == b'!' {
                            let start = (line_start + j) as u32;
                            let end = start + 2;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Avoid the use of double negation (`!!`).".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                            j += 2;
                            continue;
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
