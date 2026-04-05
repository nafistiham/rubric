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
                // Find the matching `)` for the opening `(` at the start of args_str
                // (rfind would pick the last `)` on the line, e.g. from `.gsub(...)`)
                let paren_close = {
                    let mut depth = 1usize;
                    let mut found = None;
                    for (k, b) in args_str.bytes().enumerate() {
                        if b == b'(' { depth += 1; }
                        else if b == b')' {
                            depth -= 1;
                            if depth == 0 { found = Some(k); break; }
                        }
                    }
                    found
                };
                if let Some(paren_close) = paren_close {
                    let args_inner = &args_str[..paren_close];

                    // Find the format string (first argument)
                    let fmt_str = extract_first_string(args_inner);
                    if let Some(fmt) = fmt_str {
                        // Count format specifiers
                        let specifier_count = count_format_specifiers(fmt);
                        // Count remaining arguments (after the format string).
                        // Must track parentheses/brackets to skip commas inside nested calls.
                        let remaining = args_inner.trim_start_matches(|c: char| c != ',');
                        let arg_count = if remaining.is_empty() || !remaining.starts_with(',') {
                            0
                        } else {
                            count_top_level_args(&remaining[1..])
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

/// Count top-level comma-separated arguments in `s`, respecting nested parens/brackets/braces.
fn count_top_level_args(s: &str) -> usize {
    if s.trim().is_empty() {
        return 0;
    }
    let bytes = s.as_bytes();
    let mut depth: i32 = 0;
    let mut count = 1usize;
    let mut in_string: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(d) = in_string {
            if b == b'\\' { i += 2; continue; }
            if b == d { in_string = None; }
        } else {
            match b {
                b'"' | b'\'' => { in_string = Some(b); }
                b'(' | b'[' | b'{' => { depth += 1; }
                b')' | b']' | b'}' => { if depth > 0 { depth -= 1; } }
                b',' if depth == 0 => { count += 1; }
                _ => {}
            }
        }
        i += 1;
    }
    count
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
            let mut k = i + 1;
            // Skip `%%` (literal percent)
            if bytes[k] == b'%' {
                i = k + 1;
                continue;
            }
            // Skip optional flags: `-`, `+`, ` `, `#`, `0`
            while k < n && matches!(bytes[k], b'-' | b'+' | b' ' | b'#' | b'0') {
                k += 1;
            }
            // Skip optional width (digits)
            while k < n && bytes[k].is_ascii_digit() {
                k += 1;
            }
            // Skip optional .precision
            if k < n && bytes[k] == b'.' {
                k += 1;
                while k < n && bytes[k].is_ascii_digit() {
                    k += 1;
                }
            }
            // Match conversion character
            if k < n && matches!(bytes[k], b's' | b'd' | b'f' | b'i' | b'g' | b'e' | b'x' | b'X' | b'o' | b'b' | b'p' | b'a') {
                count += 1;
                i = k + 1;
                continue;
            }
        }
        i += 1;
    }
    count
}
