use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct InfiniteLoop;

/// Returns true if `pos` in `bytes` is inside a string literal.
fn in_string_at(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => {
                // Real comment: nothing after this is code
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Returns true if `trimmed_start` starts with `while true` as a complete
/// construct — optionally followed by whitespace and/or the `do` keyword.
/// `trimmed_start` has leading spaces removed but may have trailing spaces.
fn is_while_true(trimmed_start: &str) -> bool {
    if !trimmed_start.starts_with("while true") {
        return false;
    }
    let rest = trimmed_start["while true".len()..].trim();
    // After `while true`, only nothing or `do` is valid for an infinite loop
    rest.is_empty() || rest == "do"
}

/// Returns true if `trimmed_start` starts with `until false` as a complete
/// construct — optionally followed by whitespace and/or the `do` keyword.
fn is_until_false(trimmed_start: &str) -> bool {
    if !trimmed_start.starts_with("until false") {
        return false;
    }
    let rest = trimmed_start["until false".len()..].trim();
    rest.is_empty() || rest == "do"
}

impl Rule for InfiniteLoop {
    fn name(&self) -> &'static str {
        "Style/InfiniteLoop"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Determine if `while true` or `until false` appears as the
            // leading construct on this line (after stripping indentation)
            let indent_len = line.len() - line.trim_start().len();
            let trimmed_start = line.trim_start();

            let flagged = if is_while_true(trimmed_start) {
                // Confirm the keyword position is not inside a string
                let bytes = line.as_bytes();
                !in_string_at(bytes, indent_len)
            } else if is_until_false(trimmed_start) {
                let bytes = line.as_bytes();
                !in_string_at(bytes, indent_len)
            } else {
                false
            };

            if flagged {
                let line_start = ctx.line_start_offsets[i] as usize;
                let start = (line_start + indent_len) as u32;
                // Highlight just the keyword (`while` = 5 chars, `until` = 5 chars)
                let kw_len = 5u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `Kernel#loop` for infinite loops.".into(),
                    range: TextRange::new(start, start + kw_len),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
