use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceAfterComma;

/// Returns true when a `/` at position `j` in `line_bytes` is the start of a
/// regex literal rather than the division operator.
///
/// Heuristic: `/` starts a regex when the last meaningful token before it is an
/// operator, keyword, opening bracket, or when the position is at the start of
/// expression context.  It is a division operator when the preceding token is
/// an identifier-end character, `)`, `]`, a digit, or `_`.
fn slash_starts_regex(line_bytes: &[u8], j: usize) -> bool {
    // Scan backwards for the last non-whitespace byte before j
    let mut k = j;
    loop {
        if k == 0 {
            // Nothing before — start of line — treat as regex
            return true;
        }
        k -= 1;
        let b = line_bytes[k];
        if b == b' ' || b == b'\t' {
            continue;
        }
        // Identifier end characters / closing brackets mean division
        if b.is_ascii_alphanumeric() || b == b'_' || b == b')' || b == b']' {
            return false;
        }
        // Everything else (operators, opening brackets, commas, etc.) means regex
        return true;
    }
}

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

impl Rule for SpaceAfterComma {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterComma"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc: Option<Vec<u8>> = None;
        // Carry regex state across lines for multiline `/regex/x` literals.
        let mut in_multiline_regex = false;
        // Carry %r{...} state across lines so `/` inside multiline %r bodies
        // is not misdetected as starting a new regex literal.
        let mut in_multiline_percent_regex = false;
        let mut multiline_percent_regex_depth: usize = 0;
        // Skip comma checking inside multiline %w[...], %W[...], %i[...], %I[...].
        let mut in_percent_word_array = false;
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;
        while i < n {
            let line = &lines[i];

            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                i += 1;
                continue;
            }

            // Skip lines inside a multiline %w[...] / %W[...] / %i[...] / %I[...].
            // Words in these arrays are space-separated; commas are part of words.
            if in_percent_word_array {
                // Check if the closing `]` appears on this line
                if line.contains(']') {
                    in_percent_word_array = false;
                }
                i += 1;
                continue;
            }

            // Skip pure comment lines
            if line.trim_start().starts_with('#') {
                i += 1;
                continue;
            }
            let line_start = ctx.line_start_offsets[i] as usize;
            let line_bytes = line.as_bytes();
            let mut in_string: Option<u8> = None; // None = outside, Some(delim) = inside string
            // Seed from cross-line state so multiline /regex/ and %r{} bodies are skipped.
            let mut in_regex = in_multiline_regex;
            let mut in_percent_regex = in_multiline_percent_regex; // inside %r{...}
            let mut percent_regex_depth: usize = multiline_percent_regex_depth; // brace depth inside %r{
            let mut j = 0;
            while j < line_bytes.len() {
                let b = line_bytes[j];

                // ── Inside %r{...} ─────────────────────────────────────────
                if in_percent_regex {
                    if b == b'\\' {
                        j += 2;
                        continue;
                    }
                    if b == b'{' {
                        percent_regex_depth += 1;
                        j += 1;
                        continue;
                    }
                    if b == b'}' {
                        if percent_regex_depth == 0 {
                            in_percent_regex = false;
                        } else {
                            percent_regex_depth -= 1;
                        }
                        j += 1;
                        continue;
                    }
                    // Any other byte (including comma) — skip
                    j += 1;
                    continue;
                }

                // ── Inside /regex/ ─────────────────────────────────────────
                if in_regex {
                    if b == b'\\' {
                        j += 2;
                        continue;
                    }
                    if b == b'[' {
                        // Character class — skip until closing ]
                        j += 1;
                        while j < line_bytes.len() {
                            if line_bytes[j] == b'\\' {
                                j += 2;
                                continue;
                            }
                            if line_bytes[j] == b']' {
                                j += 1;
                                break;
                            }
                            j += 1;
                        }
                        continue;
                    }
                    if b == b'/' {
                        in_regex = false;
                        j += 1;
                        continue;
                    }
                    // Any other byte (including comma) — skip
                    j += 1;
                    continue;
                }

                // ── Inside a string ────────────────────────────────────────
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; } // skip escaped char
                    Some(b'"') if b == b'#' && j + 1 < line_bytes.len() && line_bytes[j + 1] == b'{' => {
                        // String interpolation: skip #{...} block, tracking nested braces and strings
                        j += 2; // skip #{
                        let mut depth = 1usize;
                        while j < line_bytes.len() && depth > 0 {
                            let ib = line_bytes[j];
                            if ib == b'\\' { j += 2; continue; }
                            if ib == b'{' { depth += 1; j += 1; continue; }
                            if ib == b'}' {
                                depth -= 1;
                                if depth == 0 { j += 1; break; }
                                j += 1;
                                continue;
                            }
                            // Nested string inside interpolation — skip its content
                            if ib == b'"' || ib == b'\'' || ib == b'`' {
                                let id = ib;
                                j += 1;
                                while j < line_bytes.len() {
                                    if line_bytes[j] == b'\\' { j += 2; continue; }
                                    if line_bytes[j] == id { j += 1; break; }
                                    j += 1;
                                }
                                continue;
                            }
                            j += 1;
                        }
                        continue;
                    }
                    Some(delim) if b == delim => { in_string = None; }
                    Some(_) => {}

                    // ── Outside strings and regex ──────────────────────────
                    None if b == b'"' || b == b'\'' || b == b'`' => { in_string = Some(b); }
                    None if b == b'#' => break, // inline comment — stop scanning
                    // Detect %r{ percent-regex
                    None if b == b'%'
                        && j + 2 < line_bytes.len()
                        && line_bytes[j + 1] == b'r'
                        && line_bytes[j + 2] == b'{' =>
                    {
                        in_percent_regex = true;
                        percent_regex_depth = 0;
                        j += 3; // skip %r{
                        continue;
                    }
                    // Detect /regex/ start
                    None if b == b'/' && slash_starts_regex(line_bytes, j) => {
                        in_regex = true;
                        j += 1;
                        continue;
                    }
                    // Detect %w[...], %W[...], %i[...], %I[...] word arrays.
                    // Commas inside these are word characters, not separators.
                    None if b == b'%'
                        && j + 2 < line_bytes.len()
                        && matches!(line_bytes[j + 1], b'w' | b'W' | b'i' | b'I') =>
                    {
                        let open = if j + 2 < line_bytes.len() { line_bytes[j + 2] } else { 0 };
                        let close = match open {
                            b'(' => b')', b'[' => b']', b'{' => b'}', b'<' => b'>',
                            _ => 0,
                        };
                        if close != 0 {
                            j += 3; // skip %w[
                            let mut depth = 1usize;
                            while j < line_bytes.len() && depth > 0 {
                                match line_bytes[j] {
                                    b'\\' => { j += 2; }
                                    c if c == open => { depth += 1; j += 1; }
                                    c if c == close => { depth -= 1; j += 1; }
                                    _ => { j += 1; }
                                }
                            }
                            if depth > 0 {
                                // Array continues on next lines
                                in_percent_word_array = true;
                            }
                        } else {
                            j += 1;
                        }
                        continue;
                    }
                    None if b == b',' => {
                        let next = line_bytes.get(j + 1).copied();
                        if next != Some(b' ') && next != Some(b'\t') && next.is_some() {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space missing after comma.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    None => {}
                }
                j += 1;
            }

            // Persist regex state across lines (for multiline /regex/x and %r{} literals).
            in_multiline_regex = in_regex;
            in_multiline_percent_regex = in_percent_regex;
            multiline_percent_regex_depth = percent_regex_depth;

            // Detect if this line opens a heredoc (body starts on the next line)
            if in_heredoc.is_none() {
                if let Some(term) = extract_heredoc_terminator(line_bytes) {
                    in_heredoc = Some(term);
                }
            }

            i += 1;
        }
        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: TextRange::new(diag.range.start, diag.range.end),
                replacement: ", ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}
