use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SwapValues;

/// Check if a string is a simple variable name: optional `@` or `@@` prefix,
/// then one or more alphanumeric/underscore chars, with nothing else.
fn is_simple_var(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    // Allow @, @@, or plain identifier
    if chars[i] == '@' {
        i += 1;
        if i < chars.len() && chars[i] == '@' {
            i += 1;
        }
    }

    if i >= chars.len() {
        return false;
    }

    // Must have at least one identifier char after the prefix
    let mut saw_ident = false;
    while i < chars.len() {
        let c = chars[i];
        if c.is_alphanumeric() || c == '_' {
            saw_ident = true;
        } else {
            return false;
        }
        i += 1;
    }

    saw_ident
}

/// Parse an assignment line `lhs = rhs`, returning `(lhs, rhs)` trimmed strings.
/// Returns None if the line is a comment, has no ` = `, or has compound operators.
fn parse_assignment(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut i = 0;

    while i < n {
        let b = bytes[i];

        if let Some(delim) = in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
            i += 1;
            continue;
        }

        match b {
            b'"' | b'\'' | b'`' => {
                in_string = Some(b);
                i += 1;
            }
            b'#' => break, // comment
            b'=' => {
                // Must be ` = ` pattern (space-eq-space)
                if i == 0 || i + 1 >= n {
                    i += 1;
                    continue;
                }
                // Reject ==, !=, <=, >=, +=, -=, *=, /=, ||=, &&=, etc.
                let prev = bytes[i - 1];
                if matches!(prev, b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'&' | b'^') {
                    i += 1;
                    continue;
                }
                let next = bytes[i + 1];
                if next == b'=' {
                    i += 1;
                    continue;
                }
                // Must have space before and after
                if prev == b' ' && next == b' ' {
                    let lhs = trimmed[..i].trim();
                    let rhs = trimmed[i + 1..].trim();
                    return Some((lhs, rhs));
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    None
}

impl Rule for SwapValues {
    fn name(&self) -> &'static str {
        "Style/SwapValues"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // We need at least 3 lines
        if n < 3 {
            return diags;
        }

        let mut i = 0;
        while i + 2 < n {
            let line1 = lines[i];
            let line2 = lines[i + 1];
            let line3 = lines[i + 2];

            // Parse the three lines as assignments
            if let (Some((tmp, a)), Some((a2, b)), Some((b2, tmp2))) = (
                parse_assignment(line1),
                parse_assignment(line2),
                parse_assignment(line3),
            ) {
                // All six tokens must be simple variables
                if is_simple_var(tmp)
                    && is_simple_var(a)
                    && is_simple_var(a2)
                    && is_simple_var(b)
                    && is_simple_var(b2)
                    && is_simple_var(tmp2)
                {
                    // Check swap pattern:
                    // line1: tmp = a
                    // line2: a = b     (a2 == a, b is different)
                    // line3: b = tmp   (b2 == b, tmp2 == tmp)
                    if a2 == a       // line2 LHS is `a`
                        && b2 == b   // line3 LHS is `b`
                        && tmp2 == tmp // line3 RHS is `tmp`
                        && tmp != a  // tmp is different from a
                        && tmp != b  // tmp is different from b
                        && a != b    // a and b are different
                    {
                        let indent = line1.len() - line1.trim_start().len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let start = (line_start + indent) as u32;
                        let end = (line_start + line1.trim_end().len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Replace the temporary variable with a parallel assignment."
                                .into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                        // Skip past the 3 lines to avoid double-flagging
                        i += 3;
                        continue;
                    }
                }
            }

            i += 1;
        }

        diags
    }
}
