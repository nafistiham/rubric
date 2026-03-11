use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideParens;

/// If `line` opens a heredoc, return the terminator string.
fn extract_heredoc_terminator(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            i += 2;
            if i < n && (line[i] == b'-' || line[i] == b'~') { i += 1; }
            if i < n && (line[i] == b'\'' || line[i] == b'"' || line[i] == b'`') { i += 1; }
            let start = i;
            while i < n && (line[i].is_ascii_alphanumeric() || line[i] == b'_') { i += 1; }
            if i > start { return Some(line[start..i].to_vec()); }
        } else { i += 1; }
    }
    None
}

/// Skip a percent literal starting at `j` (which points at `%`).
/// Returns `(new_j, multiline_state)`:
/// - `new_j`: position after the literal if closed, or `bytes.len()` if not.
/// - `multiline_state`: `None` if closed; `Some((close, depth))` if still open
///   (depth=0 for same-char delimiters, ≥1 for bracket-style).
/// Returns `(j + 1, None)` if `%` is not a valid percent literal opener.
fn skip_percent_literal_sip(bytes: &[u8], j: usize) -> (usize, Option<(u8, usize)>) {
    let n = bytes.len();
    let mut k = j + 1;
    // Optional type letter
    if k < n
        && matches!(
            bytes[k],
            b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b's' | b'x'
        )
    {
        k += 1;
    }
    if k >= n {
        return (j + 1, None);
    }
    let open = bytes[k];
    let close = match open {
        b'{' => b'}',
        b'(' => b')',
        b'[' => b']',
        b'<' => b'>',
        b if b.is_ascii_punctuation() => b, // same-char delimiter
        _ => return (j + 1, None),
    };
    k += 1;
    if open == close {
        while k < n {
            match bytes[k] {
                b'\\' => { k += 2; }
                c if c == close => { k += 1; return (k, None); }
                _ => { k += 1; }
            }
        }
        (n, Some((close, 0)))
    } else {
        let mut depth = 1usize;
        while k < n && depth > 0 {
            match bytes[k] {
                b'\\' => { k += 2; }
                c if c == open => { depth += 1; k += 1; }
                c if c == close => { depth -= 1; k += 1; }
                _ => { k += 1; }
            }
        }
        if depth == 0 { (k, None) } else { (n, Some((close, depth))) }
    }
}

/// Skip `#{...}` interpolation starting at `j` (pointing at `#`).
/// Caller must have verified `bytes[j+1] == b'{'`.
fn skip_interpolation_sip(bytes: &[u8], start: usize) -> usize {
    let n = bytes.len();
    let mut j = start + 2;
    let mut depth = 1usize;
    while j < n && depth > 0 {
        match bytes[j] {
            b'\\' => { j += 2; }
            b'{' => { depth += 1; j += 1; }
            b'}' => { depth -= 1; j += 1; }
            b'"' | b'\'' | b'`' => {
                let delim = bytes[j];
                j += 1;
                while j < n {
                    if bytes[j] == b'\\' { j += 2; continue; }
                    if bytes[j] == delim { j += 1; break; }
                    j += 1;
                }
            }
            _ => { j += 1; }
        }
    }
    j
}

impl Rule for SpaceInsideParens {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideParens"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let mut in_heredoc: Option<Vec<u8>> = None;
        // Cross-line percent literal: Some((close_byte, depth))
        let mut in_multiline_percent: Option<(u8, usize)> = None;
        // Cross-line /regex/ state
        let mut in_multiline_regex = false;
        // Cross-line backtick string
        let mut in_multiline_string: Option<u8> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();

            // ── Heredoc body ─────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                continue;
            }

            // ── Multiline percent literal ────────────────────────────────────
            if let Some((close, depth)) = in_multiline_percent {
                let new_state = if depth == 0 {
                    let mut k = 0;
                    let mut found = false;
                    while k < len {
                        match bytes[k] {
                            b'\\' => { k += 2; }
                            c if c == close => { found = true; break; }
                            _ => { k += 1; }
                        }
                    }
                    if found { None } else { Some((close, 0usize)) }
                } else {
                    let open = match close {
                        b')' => b'(', b']' => b'[', b'}' => b'{', b'>' => b'<', _ => 0,
                    };
                    let mut d = depth;
                    let mut k = 0;
                    while k < len && d > 0 {
                        match bytes[k] {
                            b'\\' => { k += 2; }
                            c if open != 0 && c == open => { d += 1; k += 1; }
                            c if c == close => { d -= 1; k += 1; }
                            _ => { k += 1; }
                        }
                    }
                    if d == 0 { None } else { Some((close, d)) }
                };
                in_multiline_percent = new_state;
                continue;
            }

            // ── Multiline /regex/ ────────────────────────────────────────────
            if in_multiline_regex {
                let mut k = 0;
                while k < len {
                    match bytes[k] {
                        b'\\' => { k += 2; }
                        b'/' => { in_multiline_regex = false; break; }
                        _ => { k += 1; }
                    }
                }
                continue;
            }

            // ── Continuing a multiline backtick string ───────────────────────
            if let Some(delim) = in_multiline_string {
                let mut k = 0;
                while k < len {
                    match bytes[k] {
                        b'\\' => { k += 2; }
                        c if c == delim => { in_multiline_string = None; break; }
                        _ => { k += 1; }
                    }
                }
                continue;
            }

            let mut in_string: Option<u8> = None;
            let mut in_regex = false;

            let mut j = 0;
            while j < len {
                let b = bytes[j];

                // ── Inside /regex/ ───────────────────────────────────────────
                if in_regex {
                    if b == b'\\' { j += 2; continue; }
                    if b == b'/' { in_regex = false; }
                    j += 1;
                    continue;
                }

                // ── Inside a string ──────────────────────────────────────────
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(b'"') | Some(b'`')
                        if b == b'#' && j + 1 < len && bytes[j + 1] == b'{' =>
                    {
                        j = skip_interpolation_sip(bytes, j);
                        continue;
                    }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    // Backtick string
                    None if b == b'`' => {
                        j += 1;
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'#' if j + 1 < len && bytes[j + 1] == b'{' => {
                                    j = skip_interpolation_sip(bytes, j);
                                }
                                b'`' => { closed = true; j += 1; break; }
                                _ => { j += 1; }
                            }
                        }
                        if !closed {
                            in_multiline_string = Some(b'`');
                        }
                        continue;
                    }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment
                    None => {}
                }

                // ── Percent literals (any form) ──────────────────────────────
                if b == b'%' && j + 1 < len {
                    let (new_j, ml_state) = skip_percent_literal_sip(bytes, j);
                    if let Some(state) = ml_state {
                        in_multiline_percent = Some(state);
                    }
                    j = new_j;
                    continue;
                }

                // ── Regex opener ─────────────────────────────────────────────
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if matches!(prev, b'=' | b'(' | b',' | b'[' | b' ' | b'\t' | 0) {
                        in_regex = true;
                        j += 1;
                        // Check if regex closes on this line
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'/' => { closed = true; j += 1; break; }
                                _ => { j += 1; }
                            }
                        }
                        if !closed {
                            in_multiline_regex = true;
                        }
                        in_regex = false;
                        continue;
                    }
                }

                // ── `( ` — space after open paren ────────────────────────────
                if b == b'(' && j + 1 < len && bytes[j + 1] == b' ' {
                    // Skip if the space is only before a comment or end of content
                    // (e.g., `Regexp.union( # :nodoc:` or `( #Either`)
                    let mut k = j + 1;
                    while k < len && bytes[k] == b' ' {
                        k += 1;
                    }
                    if k < len && bytes[k] != b'#' {
                        let pos = (line_start + j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space after `(` detected.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                // ── ` )` — space before close paren ──────────────────────────
                if b == b')' && j > 0 && bytes[j - 1] == b' ' {
                    // Skip if `)` is the first non-space character (indented multiline close)
                    if bytes[..j].iter().any(|&c| c != b' ') {
                        let pos = (line_start + j - 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space before `)` detected.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                j += 1;
            }

            // Detect if this line opens a heredoc (body starts on the next line)
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator(bytes) {
                    in_heredoc = Some(term);
                }
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}
