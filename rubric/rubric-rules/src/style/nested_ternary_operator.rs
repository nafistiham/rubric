use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NestedTernaryOperator;

/// Return true if there is a ternary `:` in `bytes[from..]`.
///
/// A ternary `:` is a `:` that:
/// - Is not part of `=>` (preceded by `=`)
/// - Is not part of `::` (followed by `:`)
/// - Is not part of `::` (preceded by `:`)
fn has_ternary_colon(bytes: &[u8], from: usize) -> bool {
    let len = bytes.len();
    let mut i = from;
    while i < len {
        if bytes[i] == b':' {
            let prev_is_eq = i > 0 && bytes[i - 1] == b'=';
            let next_is_colon = i + 1 < len && bytes[i + 1] == b':';
            let prev_is_colon = i > 0 && bytes[i - 1] == b':';
            if !prev_is_eq && !next_is_colon && !prev_is_colon {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Check whether the line has a nested ternary operator.
///
/// A nested ternary requires *two or more* ternary `?` operators at the SAME
/// expression level on the same line.  Two ternaries that each live in their own
/// `#{...}` interpolation segment are NOT nested — each segment is its own
/// expression context.
///
/// We also skip:
/// - characters inside single-quoted strings
/// - characters inside `/regex/` literals
/// - characters inside `%r{...}` / `%r(...)` etc. percent-regex literals
/// - the comment tail starting with `#` (but not `#{`)
fn line_has_nested_ternary(line: &str) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Count of ternary `?` at the current top-level expression context.
    let mut top_level_count = 0usize;

    while i < len {
        match bytes[i] {
            // Escape sequences
            b'\\' => { i += 2; continue; }

            // Comment (but not interpolation `#{`)
            b'#' => {
                let next = bytes.get(i + 1).copied().unwrap_or(0);
                if next != b'{' {
                    break; // rest of line is a comment
                }
                // `#{` — advance past `#`, let the loop encounter `{`
                i += 1;
                continue;
            }

            // Single-quoted string: skip entirely (no interpolation, no ternaries)
            b'\'' => {
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' { i += 2; continue; }
                    if bytes[i] == b'\'' { i += 1; break; }
                    i += 1;
                }
                continue;
            }

            // Double-quoted string: scan with interpolation tracking.
            // Each `#{...}` segment counts ternaries independently.
            b'"' => {
                i += 1;
                while i < len {
                    match bytes[i] {
                        b'\\' => { i += 2; continue; }
                        b'"' => { i += 1; break; } // end of string
                        b'#' if i + 1 < len && bytes[i + 1] == b'{' => {
                            // Enter interpolation — count ternaries for this segment
                            i += 2; // skip `#{`
                            let mut interp_depth = 1usize;
                            let mut interp_ternary_count = 0usize;
                            while i < len && interp_depth > 0 {
                                match bytes[i] {
                                    b'\\' => { i += 2; continue; }
                                    b'{' => { interp_depth += 1; i += 1; }
                                    b'}' => {
                                        interp_depth -= 1;
                                        i += 1;
                                    }
                                    b'?' => {
                                        let is_ternary_pos = i > 0 && matches!(
                                            bytes[i - 1],
                                            b' ' | b'\t' | b')' | b']' | b'\'' | b'"'
                                        );
                                        if is_ternary_pos && has_ternary_colon(bytes, i + 1) {
                                            interp_ternary_count += 1;
                                        }
                                        i += 1;
                                    }
                                    _ => { i += 1; }
                                }
                            }
                            // A nested ternary inside a single interpolation counts
                            // toward the top-level total.  Two *separate* interpolations
                            // each with one ternary do NOT count as nested.
                            if interp_ternary_count >= 2 {
                                return true;
                            }
                            continue;
                        }
                        _ => { i += 1; }
                    }
                }
                continue;
            }

            // Percent literals: %r{...}, %w(...), %(string), etc.
            b'%' if i + 1 < len => {
                let mut k = i + 1;
                if k < len && matches!(bytes[k], b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b's' | b'x') {
                    k += 1;
                }
                if k < len && matches!(bytes[k], b'{' | b'(' | b'[' | b'<' | b'|' | b'!' | b'/' | b'^') {
                    let open = bytes[k];
                    let close = match open { b'{' => b'}', b'(' => b')', b'[' => b']', b'<' => b'>', c => c };
                    let paired = open != close;
                    i = k + 1;
                    let mut depth = 1usize;
                    while i < len && depth > 0 {
                        if bytes[i] == b'\\' { i += 2; continue; }
                        if paired {
                            if bytes[i] == open { depth += 1; }
                            else if bytes[i] == close { depth -= 1; }
                        } else if bytes[i] == close {
                            depth -= 1;
                        }
                        i += 1;
                    }
                    continue;
                }
                i += 1;
                continue;
            }

            // Regex literal: /pattern/
            b'/' => {
                let prev_nonws = bytes[..i].iter().rposition(|&c| c != b' ' && c != b'\t').map(|p| bytes[p]);
                let is_regex = matches!(prev_nonws, None
                    | Some(b'=') | Some(b'(') | Some(b',') | Some(b'[')
                    | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?')
                    | Some(b':') | Some(b';') | Some(b'~') | Some(b'{') | Some(b'>'))
                    || prev_nonws.map_or(false, |c| c.is_ascii_alphabetic() || c == b'_');
                if is_regex {
                    i += 1;
                    while i < len {
                        if bytes[i] == b'\\' { i += 2; continue; }
                        if bytes[i] == b'/' { i += 1; break; }
                        i += 1;
                    }
                    continue;
                }
                i += 1;
                continue;
            }

            // Ternary `?` at top level
            b'?' => {
                let is_ternary_pos = i > 0 && matches!(
                    bytes[i - 1],
                    b' ' | b'\t' | b')' | b']' | b'\'' | b'"'
                );
                if is_ternary_pos && has_ternary_colon(bytes, i + 1) {
                    top_level_count += 1;
                    if top_level_count >= 2 {
                        return true;
                    }
                }
                i += 1;
            }

            _ => { i += 1; }
        }
    }

    false
}

impl Rule for NestedTernaryOperator {
    fn name(&self) -> &'static str {
        "Style/NestedTernaryOperator"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            if line_has_nested_ternary(line) {
                let line_start = ctx.line_start_offsets[i];
                let indent = (line.len() - trimmed.len()) as u32;
                let start = line_start + indent;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Ternary operators must not be nested. Prefer if or case expressions."
                        .into(),
                    range: TextRange::new(start, start + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
