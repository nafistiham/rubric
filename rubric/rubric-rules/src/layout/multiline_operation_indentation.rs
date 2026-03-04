use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineOperationIndentation;

// Operators that can appear at end of line for continuation
const CONTINUATION_OPS: &[&str] = &["||", "&&", "**", "+=", "-=", "*=", "/=", "+", "-", "*", "/"];

fn ends_with_operator(line: &str) -> bool {
    let trimmed = line.trim_end();
    for op in CONTINUATION_OPS {
        if trimmed.ends_with(op) {
            // Make sure it's not a comment-looking thing
            // Check that op is preceded by a space or alphanumeric
            let without_op = &trimmed[..trimmed.len() - op.len()];
            if without_op.is_empty() || without_op.ends_with(' ') || without_op.ends_with(|c: char| c.is_ascii_alphanumeric()) {
                // Extra guard for `/`: if the slash count in the line is even and >= 2,
                // the trailing `/` closes a regex literal rather than being a division
                // operator. Skip flagging this line as a continuation.
                if *op == "/" && slash_count_is_even_and_paired(trimmed) {
                    return false;
                }
                return true;
            }
        }
    }
    false
}

/// Returns true when the line contains an even number of `/` characters (>= 2),
/// which means the trailing `/` is the closing delimiter of a regex literal.
///
/// This heuristic is intentionally simple: we count all `/` chars in the
/// trimmed line. Division expressions like `a / b` have one slash and never
/// appear as a trailing operator (they don't end the line without a follow-on
/// operand). Lines ending with `/` in practice are always closing a regex
/// `/pattern/` which produces an even slash count.
fn slash_count_is_even_and_paired(trimmed: &str) -> bool {
    let count = trimmed.chars().filter(|&c| c == '/').count();
    count >= 2 && count % 2 == 0
}

/// Compute the net change in open-parenthesis depth for a single source line.
/// Only counts `(` and `)` outside string literals and comments.
/// A simplified scanner: tracks single-quote and double-quote string state,
/// stops at `#` outside a string (line comment), and ignores regex chars.
fn paren_depth_delta(line: &str) -> i32 {
    let mut delta: i32 = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // Escape inside strings — skip next char
            '\\' if in_single || in_double => {
                chars.next();
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            // Line comment outside strings — stop counting
            '#' if !in_single && !in_double => break,
            '(' if !in_single && !in_double => delta += 1,
            ')' if !in_single && !in_double => delta -= 1,
            _ => {}
        }
    }
    delta
}

impl Rule for MultilineOperationIndentation {
    fn name(&self) -> &'static str {
        "Layout/MultilineOperationIndentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track open-parenthesis depth across lines.
        // When paren_depth > 0 the current line is inside a grouping expression,
        // so alignment-style continuation indentation is expected — not `current + 2`.
        let mut paren_depth: i32 = 0;

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Only check for continuation when we are NOT inside an open paren group.
            // Inside parens the expression can use alignment indentation freely.
            if paren_depth == 0 && ends_with_operator(line) && i + 1 < n {
                let current_indent = line.len() - trimmed.len();
                let next_line = &lines[i + 1];
                let next_trimmed = next_line.trim_start();

                // Skip blank lines
                if next_trimmed.is_empty() {
                    paren_depth = (paren_depth + paren_depth_delta(line)).max(0);
                    i += 1;
                    continue;
                }

                let next_indent = next_line.len() - next_trimmed.len();
                let expected_indent = current_indent + 2;

                if next_indent != expected_indent {
                    let line_start = ctx.line_start_offsets[i + 1];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Continuation line indentation ({}) should be {} (current + 2).",
                            next_indent, expected_indent
                        ),
                        range: TextRange::new(line_start, line_start + next_indent as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            paren_depth = (paren_depth + paren_depth_delta(line)).max(0);
            i += 1;
        }

        diags
    }
}
