use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ArrayFirstLast;

/// Returns true if the byte is a valid "method access" predecessor:
/// alphanumeric, `_`, `)`, or `]` — indicating this `[0]` is a subscript,
/// not the start of a literal array.
fn is_access_predecessor(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b')' || b == b']'
}

impl Rule for ArrayFirstLast {
    fn name(&self) -> &'static str {
        "Style/ArrayFirstLast"
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
                    b'#' => break, // inline comment
                    b'[' => {
                        // Check for [0] pattern
                        if j + 2 < n && bytes[j + 1] == b'0' && bytes[j + 2] == b']' {
                            let preceded_by_access = j > 0 && is_access_predecessor(bytes[j - 1]);
                            if preceded_by_access {
                                let start = (line_start + j) as u32;
                                let end = start + 3; // [0]
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Prefer `Array#first` over `Array#[0]`.".into(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += 3;
                                continue;
                            }
                        }
                        // Check for [-1] pattern
                        if j + 3 < n
                            && bytes[j + 1] == b'-'
                            && bytes[j + 2] == b'1'
                            && bytes[j + 3] == b']'
                        {
                            let preceded_by_access = j > 0 && is_access_predecessor(bytes[j - 1]);
                            if preceded_by_access {
                                let start = (line_start + j) as u32;
                                let end = start + 4; // [-1]
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Prefer `Array#last` over `Array#[-1]`.".into(),
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
