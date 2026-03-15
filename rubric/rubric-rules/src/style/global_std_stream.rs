use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct GlobalStdStream;

impl Rule for GlobalStdStream {
    fn name(&self) -> &'static str {
        "Style/GlobalStdStream"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip full-line comments
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
                        j += 1;
                    }
                    b'#' => break, // inline comment — stop scanning
                    b'$' => {
                        // Check for $stdout or $stderr
                        let rest = &line[j..];
                        if rest.starts_with("$stdout") {
                            // Ensure it's not followed by alphanumeric/underscore (longer name)
                            let after = j + 7;
                            let is_exact = after >= n
                                || !(bytes[after].is_ascii_alphanumeric() || bytes[after] == b'_');
                            if is_exact {
                                let start = (line_start + j) as u32;
                                let end = (line_start + j + 7) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use STDOUT instead of $stdout.".into(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += 7;
                                continue;
                            }
                        } else if rest.starts_with("$stderr") {
                            let after = j + 7;
                            let is_exact = after >= n
                                || !(bytes[after].is_ascii_alphanumeric() || bytes[after] == b'_');
                            if is_exact {
                                let start = (line_start + j) as u32;
                                let end = (line_start + j + 7) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use STDERR instead of $stderr.".into(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += 7;
                                continue;
                            }
                        }
                        j += 1;
                    }
                    _ => {
                        j += 1;
                    }
                }
            }
        }

        diags
    }
}
