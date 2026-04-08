use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingCommaInArguments;

/// Skip over a string literal starting at `bytes[i]` (which is `"` or `'`).
/// Returns the index just past the closing delimiter.
fn skip_string(bytes: &[u8], mut i: usize) -> usize {
    let delim = bytes[i];
    i += 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' { i += 2; continue; }
        if bytes[i] == delim { return i + 1; }
        // Skip interpolation #{...} inside double-quoted strings
        if delim == b'"' && bytes[i] == b'#' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            i += 2;
            let mut depth = 1usize;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] == b'{' { depth += 1; }
                else if bytes[i] == b'}' { depth -= 1; }
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    i
}

/// Skip over a `/regex/` literal starting at `bytes[i]` (which is `/`).
/// Returns the index just past the closing `/`.
fn skip_regex(bytes: &[u8], mut i: usize) -> usize {
    i += 1; // skip opening /
    while i < bytes.len() {
        if bytes[i] == b'\\' { i += 2; continue; }
        if bytes[i] == b'/' { return i + 1; }
        i += 1;
    }
    i
}

/// Skip over a percent literal `%(...)`  / `%r{...}` / `%w[...]` etc.
/// `i` points at `%`. Returns index past the closing delimiter.
fn skip_percent_literal(bytes: &[u8], mut i: usize) -> usize {
    i += 1; // skip `%`
    if i < bytes.len() && matches!(bytes[i], b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b's' | b'x') {
        i += 1; // skip type char
    }
    if i >= bytes.len() { return i; }
    let open = bytes[i];
    let close = match open { b'{' => b'}', b'(' => b')', b'[' => b']', b'<' => b'>', c => c };
    let paired = open != close;
    i += 1;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'\\' { i += 2; continue; }
        if paired {
            if bytes[i] == open { depth += 1; }
            else if bytes[i] == close { depth -= 1; }
        } else if bytes[i] == close {
            depth -= 1;
        }
        i += 1;
    }
    i
}

/// Returns true if `bytes[pos]` is a `/` that starts a regex literal.
fn is_regex_start(bytes: &[u8], pos: usize) -> bool {
    let prev_nonws = bytes[..pos].iter().rposition(|&c| c != b' ' && c != b'\t').map(|p| bytes[p]);
    matches!(prev_nonws, None
        | Some(b'=') | Some(b'(') | Some(b',') | Some(b'[')
        | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?')
        | Some(b':') | Some(b';') | Some(b'~') | Some(b'{') | Some(b'>'))
        || prev_nonws.map_or(false, |c| c.is_ascii_alphabetic() || c == b'_')
}

impl Rule for TrailingCommaInArguments {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInArguments"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_end();
            let bytes = trimmed.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Single-line check: look for , followed by optional whitespace then )
            // Skip characters inside strings, regexes, and percent literals.
            let mut j = 0;
            while j < bytes.len() {
                match bytes[j] {
                    b'#' => {
                        // Comment start (not interpolation — we're not inside a string here)
                        let next = bytes.get(j + 1).copied().unwrap_or(0);
                        if next != b'{' { break; }
                        j += 1; // let next iteration handle `{`
                    }
                    b'\'' | b'"' => {
                        j = skip_string(bytes, j);
                    }
                    b'/' if is_regex_start(bytes, j) => {
                        j = skip_regex(bytes, j);
                    }
                    b'%' if j + 1 < bytes.len() && matches!(bytes[j + 1], b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b's' | b'x' | b'(' | b'[' | b'{' | b'<') => {
                        j = skip_percent_literal(bytes, j);
                    }
                    b',' => {
                        let rest = &bytes[j + 1..];
                        let spaces: usize = rest
                            .iter()
                            .take_while(|&&b| b == b' ' || b == b'\t')
                            .count();
                        if rest.get(spaces).copied() == Some(b')') {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Trailing comma in argument list.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                        j += 1;
                    }
                    _ => { j += 1; }
                }
            }

            // Multi-line check: line ends with `,` and next non-empty line is just `)`
            // Skip if the line appears to be inside a regex or string (heuristic: starts with /)
            if trimmed.ends_with(',') {
                let next_non_empty = ctx.lines[i + 1..].iter().find(|l| !l.trim().is_empty());
                if next_non_empty.map(|l| l.trim() == ")").unwrap_or(false) {
                    let comma_pos = (line_start + trimmed.len() - 1) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Trailing comma in argument list.".into(),
                        range: TextRange::new(comma_pos, comma_pos + 1),
                        severity: Severity::Warning,
                    });
                }
            }
        }
        diags
    }
}
