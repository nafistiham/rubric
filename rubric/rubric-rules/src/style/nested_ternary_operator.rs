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
/// "Nested" means one ternary lives inside the true- or false-branch of another,
/// e.g. `a ? b ? c : d : e` or `a ? b : c ? d : e`.
///
/// Multiple ternaries at the *same* level (parallel), e.g.
/// `params.key?(:x) ? foo : nil, params.key?(:y) ? bar : nil`, are NOT nested.
///
/// We use a stack to track open ternaries and brackets ({}/()[]):
/// - Ternary `?` → push Ternary onto stack; if stack now has 2 Ternary entries → nested
/// - Ternary `:` (only when top of stack is a Ternary) → pop stack
/// - `{`/`(`/`[` → push Bracket; shields inner `:` from being treated as ternary
/// - `}`/`)`/`]` → pop until a Bracket is removed (unclosed ternaries inside close too)
///
/// We also skip:
/// - characters inside single-quoted strings
/// - characters inside `/regex/` literals
/// - characters inside `%r{...}` / `%r(...)` etc. percent-regex literals
/// - `#{...}` interpolation segments (each is its own independent context)
/// - the comment tail starting with `#` (but not `#{`)
fn line_has_nested_ternary(line: &str) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Stack entries: true = Ternary open, false = Bracket open ({, (, [)
    let mut stack: Vec<bool> = Vec::new();

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
                // `#{` — enter interpolation as an independent ternary context
                i += 2; // skip `#{`
                if check_interpolation_nested_ternary(bytes, &mut i) {
                    return true;
                }
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
            // Each `#{...}` segment is its own independent ternary context.
            b'"' => {
                i += 1;
                while i < len {
                    match bytes[i] {
                        b'\\' => { i += 2; continue; }
                        b'"' => { i += 1; break; } // end of string
                        b'#' if i + 1 < len && bytes[i + 1] == b'{' => {
                            i += 2; // skip `#{`
                            if check_interpolation_nested_ternary(bytes, &mut i) {
                                return true;
                            }
                        }
                        _ => { i += 1; }
                    }
                }
                continue;
            }

            // Percent literals: %r{...}, %w(...), %(string), etc. — skip entirely
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

            // Opening brackets: shield inner `:` from being treated as ternary colon
            b'{' | b'(' | b'[' => {
                stack.push(false); // false = bracket
                i += 1;
            }

            // Closing brackets: pop until a bracket entry is removed
            b'}' | b')' | b']' => {
                while let Some(top) = stack.last() {
                    let is_bracket = !top;
                    stack.pop();
                    if is_bracket { break; }
                }
                i += 1;
            }

            // Ternary `?`
            b'?' => {
                let is_ternary_pos = i > 0 && matches!(
                    bytes[i - 1],
                    b' ' | b'\t' | b')' | b']' | b'\'' | b'"'
                );
                if is_ternary_pos && has_ternary_colon(bytes, i + 1) {
                    stack.push(true); // true = ternary
                    // Count only Ternary entries in the stack
                    let ternary_depth = stack.iter().filter(|&&e| e).count();
                    if ternary_depth >= 2 {
                        return true;
                    }
                }
                i += 1;
            }

            // `:` — close the innermost open ternary if it's on top of the stack
            b':' => {
                let prev_is_eq = i > 0 && bytes[i - 1] == b'=';
                let next_is_colon = i + 1 < len && bytes[i + 1] == b':';
                let prev_is_colon = i > 0 && bytes[i - 1] == b':';
                if !prev_is_eq && !next_is_colon && !prev_is_colon {
                    if stack.last() == Some(&true) {
                        stack.pop(); // close the innermost ternary
                    }
                }
                i += 1;
            }

            _ => { i += 1; }
        }
    }

    false
}

/// Scan through a `#{...}` interpolation starting just after the `{`.
/// Advances `*i` to just past the closing `}`.
/// Returns true if the interpolation itself contains a nested ternary.
fn check_interpolation_nested_ternary(bytes: &[u8], i: &mut usize) -> bool {
    let len = bytes.len();
    let mut depth = 1usize;
    // Stack for ternary nesting inside the interpolation
    let mut stack: Vec<bool> = Vec::new();

    while *i < len && depth > 0 {
        match bytes[*i] {
            b'\\' => { *i += 2; continue; }
            b'{' => { depth += 1; stack.push(false); *i += 1; }
            b'}' => {
                depth -= 1;
                if depth > 0 {
                    while let Some(top) = stack.last() {
                        let is_bracket = !top;
                        stack.pop();
                        if is_bracket { break; }
                    }
                }
                *i += 1;
            }
            b'(' | b'[' => { stack.push(false); *i += 1; }
            b')' | b']' => {
                while let Some(top) = stack.last() {
                    let is_bracket = !top;
                    stack.pop();
                    if is_bracket { break; }
                }
                *i += 1;
            }
            b'?' => {
                let is_ternary_pos = *i > 0 && matches!(
                    bytes[*i - 1],
                    b' ' | b'\t' | b')' | b']' | b'\'' | b'"'
                );
                if is_ternary_pos && has_ternary_colon(bytes, *i + 1) {
                    stack.push(true);
                    let ternary_depth = stack.iter().filter(|&&e| e).count();
                    if ternary_depth >= 2 { return true; }
                }
                *i += 1;
            }
            b':' => {
                let prev_is_eq = *i > 0 && bytes[*i - 1] == b'=';
                let next_is_colon = *i + 1 < len && bytes[*i + 1] == b':';
                let prev_is_colon = *i > 0 && bytes[*i - 1] == b':';
                if !prev_is_eq && !next_is_colon && !prev_is_colon {
                    if stack.last() == Some(&true) {
                        stack.pop();
                    }
                }
                *i += 1;
            }
            _ => { *i += 1; }
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
