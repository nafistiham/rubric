use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FormatParameterMismatch;

impl Rule for FormatParameterMismatch {
    fn name(&self) -> &'static str {
        "Lint/FormatParameterMismatch"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let call_start = if trimmed.starts_with("sprintf(") {
                Some("sprintf(")
            } else if trimmed.starts_with("format(") {
                Some("format(")
            } else {
                None
            };

            if let Some(prefix) = call_start {
                let args_str = &trimmed[prefix.len()..];
                if let Some(paren_close) = args_str.rfind(')') {
                    let args_inner = &args_str[..paren_close];

                    // Find the format string (first argument)
                    let fmt_str = extract_first_string(args_inner);
                    if let Some(fmt) = fmt_str {
                        // Count format specifiers
                        let specifier_count = count_format_specifiers(fmt);
                        // Count remaining arguments (after the format string)
                        let remaining = args_inner.trim_start_matches(|c: char| c != ',');
                        let arg_count = if remaining.is_empty() || !remaining.starts_with(',') {
                            0
                        } else {
                            remaining[1..].split(',').count()
                        };

                        if specifier_count != arg_count {
                            let indent = line.len() - trimmed.len();
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let pos = (line_start + indent) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Format string has {} specifier(s) but {} argument(s) given.",
                                    specifier_count, arg_count
                                ),
                                range: TextRange::new(pos, pos + trimmed.len() as u32),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        diags
    }
}

fn extract_first_string(s: &str) -> Option<&str> {
    let s = s.trim();
    if s.starts_with('"') {
        let end = s[1..].find('"')? + 1;
        Some(&s[1..end])
    } else if s.starts_with('\'') {
        let end = s[1..].find('\'')? + 1;
        Some(&s[1..end])
    } else {
        None
    }
}

fn count_format_specifiers(fmt: &str) -> usize {
    let bytes = fmt.as_bytes();
    let n = bytes.len();
    let mut count = 0;
    let mut i = 0;
    while i < n {
        if bytes[i] == b'%' && i + 1 < n {
            let next = bytes[i + 1];
            if matches!(next, b's' | b'd' | b'f' | b'i' | b'g' | b'e' | b'x' | b'o' | b'b' | b'p') {
                count += 1;
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    count
}
