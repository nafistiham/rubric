use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineTernaryOperator;

impl Rule for MultilineTernaryOperator {
    fn name(&self) -> &'static str {
        "Style/MultilineTernaryOperator"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let code = strip_inline_comment(line.trim());
            let code = code.trim_end();

            // A multiline ternary's condition line ends with ` ?` (space + question mark).
            // This distinguishes it from predicate methods which end with `foo?` (no space).
            if !code.ends_with(" ?") {
                continue;
            }

            // The `?` must not be inside a string literal.
            // strip_inline_comment already stripped the comment portion; now verify
            // the trailing ` ?` is not inside a string by checking for open string state.
            if ends_in_string_context(code) {
                continue;
            }

            // Find the byte offset of the `?` character at end of this line.
            let line_start = ctx.line_start_offsets[i];
            // Trim from the original (non-trimmed) line to find the `?` position.
            let raw_line = ctx.lines[i];
            let q_offset = find_trailing_ternary_question(raw_line);
            if let Some(col) = q_offset {
                let start = line_start + col as u32;
                let end = start + 1;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Avoid multi-line ternary operators, use `if` or `unless` instead."
                        .into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Return the byte column of the trailing ` ?` in a raw line (the `?` character),
/// or None if the line does not end with a ternary question mark.
fn find_trailing_ternary_question(line: &str) -> Option<usize> {
    let code = strip_inline_comment(line.trim());
    let code = code.trim_end();

    if !code.ends_with(" ?") {
        return None;
    }

    // Position of `?` within the trimmed code portion
    let q_in_code = code.len() - 1;

    // Map back to position in the original line.
    // The trimmed prefix skips leading whitespace.
    let leading_ws = line.len() - line.trim_start().len();
    Some(leading_ws + q_in_code)
}

/// Return true if the code (after comment stripping) ends while still inside
/// a string literal — meaning the trailing `?` is part of a string, not an operator.
fn ends_in_string_context(code: &str) -> bool {
    let bytes = code.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut j = 0;
    while j < n {
        let b = bytes[j];
        if let Some(delim) = in_string {
            if b == b'\\' {
                j += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
        } else {
            match b {
                b'"' | b'\'' | b'`' => in_string = Some(b),
                _ => {}
            }
        }
        j += 1;
    }
    in_string.is_some()
}

/// Strip inline comment from a line, respecting string literals.
fn strip_inline_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut j = 0;
    while j < n {
        let b = bytes[j];
        if let Some(delim) = in_string {
            if b == b'\\' {
                j += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
        } else {
            match b {
                b'"' | b'\'' | b'`' => in_string = Some(b),
                b'#' => return &line[..j],
                _ => {}
            }
        }
        j += 1;
    }
    line
}
