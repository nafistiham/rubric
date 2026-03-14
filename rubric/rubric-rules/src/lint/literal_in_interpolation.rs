use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct LiteralInInterpolation;

impl Rule for LiteralInInterpolation {
    fn name(&self) -> &'static str {
        "Lint/LiteralInInterpolation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip pure comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                // Only scan inside double-quoted strings
                if b != b'"' {
                    j += 1;
                    continue;
                }

                // We found the opening `"` — scan the string body
                j += 1; // consume `"`
                while j < n {
                    let c = bytes[j];
                    match c {
                        b'\\' => {
                            // Escape sequence — skip two bytes
                            j += 2;
                        }
                        b'"' => {
                            // End of double-quoted string
                            j += 1;
                            break;
                        }
                        b'#' if j + 1 < n && bytes[j + 1] == b'{' => {
                            // Interpolation start: `#{`
                            let interp_start = j; // position of `#`
                            j += 2; // skip `#` and `{`

                            // Collect content until the matching `}`
                            let content_start = j;
                            let mut depth = 1usize;
                            while j < n && depth > 0 {
                                match bytes[j] {
                                    b'{' => depth += 1,
                                    b'}' => depth -= 1,
                                    _ => {}
                                }
                                if depth > 0 {
                                    j += 1;
                                }
                            }
                            let content_end = j;
                            // `j` now points at the closing `}` (depth == 0)
                            if j < n {
                                j += 1; // consume `}`
                            }

                            let content = std::str::from_utf8(&bytes[content_start..content_end])
                                .unwrap_or("")
                                .trim();

                            if is_literal(content) {
                                let start = (line_start + interp_start) as u32;
                                // end covers the full `#{...}` including closing `}`
                                let end = (line_start + j) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Literal interpolation detected.".into(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                        _ => {
                            j += 1;
                        }
                    }
                }
            }
        }

        diags
    }
}

/// Returns true if `content` (the trimmed interior of `#{...}`) is a literal value:
/// - Integer: all digits, optionally with `_` separators (e.g. `42`, `1_000`)
/// - Float: digits with a single `.` (e.g. `3.14`)
/// - `nil`, `true`, `false`
/// - Symbol: starts with `:` followed by word characters (e.g. `:foo`, `:bar_baz`)
/// - Single-quoted string: starts and ends with `'`
/// - Double-quoted string: starts and ends with `"`
fn is_literal(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }

    // Keyword literals
    if matches!(content, "nil" | "true" | "false") {
        return true;
    }

    // Integer (digits with optional underscore separators)
    if content
        .chars()
        .all(|c| c.is_ascii_digit() || c == '_')
        && content.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
    {
        return true;
    }

    // Float (digits, exactly one dot, digits)
    if is_float_literal(content) {
        return true;
    }

    // Symbol: `:identifier` (`:foo`, `:bar_baz`)
    if content.starts_with(':') {
        let rest = &content[1..];
        if !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
        {
            return true;
        }
    }

    // Single-quoted string inside interpolation: `'...'`
    if content.starts_with('\'') && content.ends_with('\'') && content.len() >= 2 {
        return true;
    }

    // Double-quoted string inside interpolation: `"..."`
    if content.starts_with('"') && content.ends_with('"') && content.len() >= 2 {
        return true;
    }

    false
}

/// Returns true if `s` looks like a float literal (e.g. `3.14`, `0.5`).
fn is_float_literal(s: &str) -> bool {
    let dot_count = s.chars().filter(|&c| c == '.').count();
    if dot_count != 1 {
        return false;
    }
    let parts: Vec<&str> = s.splitn(2, '.').collect();
    let integer_part = parts[0];
    let frac_part = parts[1];
    !integer_part.is_empty()
        && integer_part.chars().all(|c| c.is_ascii_digit())
        && !frac_part.is_empty()
        && frac_part.chars().all(|c| c.is_ascii_digit())
}
