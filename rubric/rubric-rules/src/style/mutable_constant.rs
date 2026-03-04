use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MutableConstant;

impl Rule for MutableConstant {
    fn name(&self) -> &'static str {
        "Style/MutableConstant"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        // Honour `# frozen_string_literal: true` — strings are already immutable in that file.
        let frozen_file = ctx.source.contains("# frozen_string_literal: true");

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Match: CONSTANT_NAME = [ or { or "
            // Constant starts with uppercase letter
            let bytes = trimmed.as_bytes();
            if bytes.is_empty() || !bytes[0].is_ascii_uppercase() {
                continue;
            }

            // Find `=`
            let Some(eq_pos) = trimmed.find(" = ") else { continue; };

            // The LHS must be a bare constant name — no receiver before a dot.
            // `Devise.mailer = ...` is a setter method call, not a constant
            // assignment.  A Ruby constant is an unqualified uppercase
            // identifier, e.g. `FOO`, `MY_CONST`.
            let lhs = trimmed[..eq_pos].trim_end();
            if !is_bare_constant(lhs) {
                continue;
            }

            let rhs = trimmed[eq_pos + 3..].trim_start();

            // Check if rhs is a mutable literal (array, hash, string) without .freeze.
            // String constants are exempt when the file has `# frozen_string_literal: true`.
            let is_string = rhs.starts_with('"') || rhs.starts_with('\'');
            let opens_array = rhs.starts_with('[');
            let opens_hash = rhs.starts_with('{');
            let is_mutable = opens_array || opens_hash || (is_string && !frozen_file);

            if !is_mutable {
                continue;
            }

            // Determine if `.freeze` is present. For single-line assignments it
            // is on the same line. For multi-line array/hash literals the
            // closing `]` or `}` is on a later line — scan forward to find it.
            let is_frozen = if opens_array {
                collection_is_frozen(rhs, i, lines, '[', ']')
            } else if opens_hash {
                collection_is_frozen(rhs, i, lines, '{', '}')
            } else {
                // Single-line string
                rhs.contains(".freeze")
            };

            if !is_frozen {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                // Point to the start of the constant name
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Mutable object assigned to constant. Use `.freeze` to make it immutable.".into(),
                    range: TextRange::new(
                        (line_start + indent) as u32,
                        (line_start + indent + eq_pos) as u32,
                    ),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Returns `true` if the collection literal that starts on `rhs` (at line
/// `start_idx` of `lines`) should be considered "safe" — either because it is
/// followed by `.freeze`, or because its value is the result of a non-freeze
/// method chain (e.g. `[a, b].join('x')`) in which case the assigned value is
/// not a raw mutable collection.
///
/// For single-line collections the check is performed on the opening line.
/// For multi-line collections the closing bracket appears on a later line;
/// we scan forward until the bracket depth returns to 0 and then check
/// whether that closing line contains `.freeze` or a chained method call.
fn collection_is_frozen(
    rhs: &str,
    start_idx: usize,
    lines: &[&str],
    open_char: char,
    close_char: char,
) -> bool {
    // Fast path: .freeze on the opening line (single-line literal).
    if rhs.contains(".freeze") {
        return true;
    }

    // Compute the net depth contributed by the opening line.
    let mut depth = count_net_depth(rhs, open_char, close_char);

    if depth <= 0 {
        // The literal is closed on the same line. `.freeze` is not present
        // (checked above). Check whether a non-freeze method is chained
        // after the closing bracket — if so, the assigned value is the result
        // of that method call, not a raw collection; we don't flag it.
        return closing_line_has_method_chain(rhs, close_char);
    }

    // Multi-line literal: scan subsequent lines until depth returns to 0.
    for line in lines.iter().skip(start_idx + 1) {
        depth += count_net_depth(line, open_char, close_char);
        if depth <= 0 {
            // This is the line that closes the collection.
            // Accept both `.freeze` and any other method chain (which implies
            // the constant receives the return value of that method, not the
            // raw collection).
            return line.contains(".freeze") || closing_line_has_method_chain(line, close_char);
        }
    }

    // Unclosed literal (likely a parse error in the source) — don't flag it.
    false
}

/// Returns `true` if `line` contains a method call chained after the
/// (last) `close_char` on that line.  This indicates the constant is assigned
/// the result of a method call, not the raw collection, so it should not be
/// flagged by MutableConstant.
///
/// Example: `].join("; ")` → true; `]` alone → false; `].freeze` → true
/// (handled by caller via `.freeze` check first).
fn closing_line_has_method_chain(line: &str, close_char: char) -> bool {
    // Find the last occurrence of close_char in the line and see whether
    // a `.method_name` pattern follows it (after optional whitespace).
    let close_str: &[char] = &[close_char];
    let Some(pos) = line.rfind(close_str) else { return false; };
    let after = line[pos + close_char.len_utf8()..].trim_start();
    // Anything starting with `.` after the closing bracket is a method chain.
    after.starts_with('.')
}

/// Counts the net change in bracket depth for `s`, where each unquoted
/// occurrence of `open_char` adds 1 and each unquoted `close_char` subtracts
/// 1.  Simple string-quote tracking is applied so that brackets inside string
/// literals are ignored.  Comment characters (`#` outside strings) stop
/// counting for the rest of the line.
fn count_net_depth(s: &str, open_char: char, close_char: char) -> i32 {
    let mut depth: i32 = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_single_quote || in_double_quote => {
                // Skip the next character — it is escaped.
                chars.next();
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '#' if !in_single_quote && !in_double_quote => {
                // Rest of line is a Ruby comment.
                break;
            }
            c if !in_single_quote && !in_double_quote => {
                if c == open_char {
                    depth += 1;
                } else if c == close_char {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }
    depth
}

/// Returns true iff `s` is a bare Ruby constant name: starts with an
/// uppercase letter, contains only uppercase letters, digits, and underscores,
/// and has no `.` (which would indicate a receiver or namespace).
fn is_bare_constant(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Any dot means this is either a method call or a scoped constant access
    // with a receiver — not a bare local constant assignment.
    if s.contains('.') {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}
