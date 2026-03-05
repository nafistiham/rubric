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

/// Count `/` characters that appear outside of string literals and comments.
/// This avoids counting path separators inside strings (e.g. `"config/routes.rb"`).
fn code_slash_count(line: &str) -> usize {
    let mut count = 0;
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'\\' if in_single || in_double => { i += 2; continue; }
            b'\'' if !in_double => { in_single = !in_single; }
            b'"' if !in_single => { in_double = !in_double; }
            b'#' if !in_single && !in_double => break,
            b'/' if !in_single && !in_double => { count += 1; }
            _ => {}
        }
        i += 1;
    }
    count
}

/// Returns true when the line contains an even number of code-level `/` characters
/// (>= 2), which means the trailing `/` closes a regex literal rather than being
/// a division operator.  Slashes inside string literals are excluded from the count
/// so that `assert_file "path/to/file", /regex/` is correctly identified as
/// ending with a regex (2 code slashes) rather than a division (3 total slashes).
fn slash_count_is_even_and_paired(trimmed: &str) -> bool {
    let count = code_slash_count(trimmed);
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

        // Track the base indentation of the first line in a continuation chain.
        // For a chain `A && \n B && \n C`, all continuation lines (B, C, …) must be
        // at base+2 where base is A's indentation — not escalating by 2 each time.
        let mut chain_base_indent: Option<usize> = None;

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
                    chain_base_indent = None;
                    paren_depth = (paren_depth + paren_depth_delta(line)).max(0);
                    i += 1;
                    continue;
                }

                // Establish chain base on the first line of the chain; subsequent
                // lines in the same chain reuse the same base so indentation does
                // not escalate for each additional `&&`/`||` operator.
                let base_indent = *chain_base_indent.get_or_insert(current_indent);
                let expected_indent = base_indent + 2;

                let next_indent = next_line.len() - next_trimmed.len();

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
            } else {
                // Line does not continue with an operator — chain is over.
                chain_base_indent = None;
            }

            paren_depth = (paren_depth + paren_depth_delta(line)).max(0);
            i += 1;
        }

        diags
    }
}
