use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceAfterComma;

/// Returns true if `word` is a Ruby keyword that may be followed by a regex literal.
fn is_regex_keyword(word: &[u8]) -> bool {
    matches!(
        word,
        b"if" | b"unless" | b"while" | b"until" | b"and" | b"or" | b"not"
            | b"return" | b"do" | b"then" | b"else" | b"elsif" | b"case"
            | b"when" | b"rescue" | b"in" | b"end" | b"yield"
    )
}

/// Returns true when the `/` at position `j` starts a regex literal rather
/// than acting as the division operator.
///
/// Heuristic: scan back past whitespace; if the preceding token is a closing
/// bracket/paren or a digit it's division; if it's a keyword it's regex;
/// otherwise (operators, opening brackets, ...) it's regex.
fn slash_starts_regex(line_bytes: &[u8], j: usize) -> bool {
    let mut k = j;
    // Skip whitespace before `/`
    while k > 0 && (line_bytes[k - 1] == b' ' || line_bytes[k - 1] == b'\t') {
        k -= 1;
    }
    if k == 0 {
        return true; // start of meaningful content → regex
    }
    let prev = line_bytes[k - 1];
    // `)` or `]` → division (result of expression)
    if prev == b')' || prev == b']' {
        return false;
    }
    // Digit → division: `42 / n`
    if prev.is_ascii_digit() {
        return false;
    }
    // Identifier or keyword end char
    if prev.is_ascii_alphabetic() || prev == b'_' {
        // Scan back to find the whole identifier / keyword
        let word_end = k;
        let mut word_start = k;
        while word_start > 0
            && (line_bytes[word_start - 1].is_ascii_alphanumeric()
                || line_bytes[word_start - 1] == b'_')
        {
            word_start -= 1;
        }
        let word = &line_bytes[word_start..word_end];
        // If it's a Ruby keyword → regex; otherwise identifier → division
        return is_regex_keyword(word);
    }
    // Anything else (operators, opening brackets, ...) → regex
    true
}

/// Skip a percent literal starting at `j` (which points at `%`).
///
/// Handles every form: `%r`, `%q`, `%Q`, `%w`, `%W`, `%i`, `%I`, `%s`, `%x`,
/// and bare `%` followed by any punctuation delimiter.
///
/// Returns `(new_j, multiline_state)`:
/// - `new_j`         — position after the literal if it closed on this line,
///                     or `bytes.len()` if it extends to the next line.
/// - `multiline_state` — `None` if closed; `Some((close, depth))` if still open
///   (depth = 0 for same-char delimiters, ≥ 1 for bracket-style).
///
/// Returns `(j + 1, None)` when `%` does not introduce a valid percent literal
/// (e.g. the modulo operator `% 2`).
fn skip_percent_literal(bytes: &[u8], j: usize) -> (usize, Option<(u8, usize)>) {
    let n = bytes.len();
    let mut k = j + 1; // skip `%`

    // Optional type letter (r q Q w W i I s x)
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
        // Any other ASCII punctuation is a same-char delimiter
        b if b.is_ascii_punctuation() => b,
        // Spaces, digits, alphanumeric → not a percent literal (modulo operator)
        _ => return (j + 1, None),
    };
    k += 1; // skip opening delimiter

    if open == close {
        // ── Same-char delimiter (e.g. `%r/.../ `, `%|...|`) ──────────────────
        while k < n {
            match bytes[k] {
                b'\\' => {
                    k += 2;
                }
                c if c == close => {
                    k += 1;
                    return (k, None);
                }
                _ => {
                    k += 1;
                }
            }
        }
        // Literal not closed on this line
        (n, Some((close, 0)))
    } else {
        // ── Bracket-style delimiter (e.g. `%r[...]`, `%(...)`) ───────────────
        let mut depth = 1usize;
        while k < n && depth > 0 {
            match bytes[k] {
                b'\\' => {
                    k += 2;
                }
                c if c == open => {
                    depth += 1;
                    k += 1;
                }
                c if c == close => {
                    depth -= 1;
                    k += 1;
                }
                _ => {
                    k += 1;
                }
            }
        }
        if depth == 0 {
            (k, None)
        } else {
            (n, Some((close, depth)))
        }
    }
}

/// If `line` opens a heredoc, return the terminator string.
fn extract_heredoc_terminator(line: &[u8]) -> Option<Vec<u8>> {
    let n = line.len();
    let mut i = 0;
    while i + 1 < n {
        if line[i] == b'<' && line[i + 1] == b'<' {
            i += 2;
            if i < n && (line[i] == b'-' || line[i] == b'~') {
                i += 1;
            }
            if i < n && (line[i] == b'\'' || line[i] == b'"' || line[i] == b'`') {
                i += 1;
            }
            let start = i;
            while i < n && (line[i].is_ascii_alphanumeric() || line[i] == b'_') {
                i += 1;
            }
            if i > start {
                return Some(line[start..i].to_vec());
            }
        } else {
            i += 1;
        }
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
        // Cross-line percent-literal state: Some((close_byte, depth))
        // depth = 0  → same-char delimiter (e.g. `%r/.../`)
        // depth ≥ 1  → bracket-style (e.g. `%r[...]`, `%(...)`)
        let mut in_multiline_percent: Option<(u8, usize)> = None;
        // Cross-line /regex/x state
        let mut in_multiline_regex = false;
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;
        while i < n {
            let line = &lines[i];

            // ── Heredoc body ─────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim().as_bytes() == term.as_slice() {
                    in_heredoc = None;
                }
                i += 1;
                continue;
            }

            // ── Multiline percent-literal body ───────────────────────────────
            if let Some((close, depth)) = in_multiline_percent {
                let bytes = line.as_bytes();
                let new_state = if depth == 0 {
                    // Same-char delimiter: scan until unescaped close
                    let mut k = 0;
                    let mut found = false;
                    while k < bytes.len() {
                        match bytes[k] {
                            b'\\' => {
                                k += 2;
                            }
                            c if c == close => {
                                found = true;
                                break;
                            }
                            _ => {
                                k += 1;
                            }
                        }
                    }
                    if found { None } else { Some((close, 0usize)) }
                } else {
                    // Bracket-style: track nesting depth
                    let open = match close {
                        b')' => b'(',
                        b']' => b'[',
                        b'}' => b'{',
                        b'>' => b'<',
                        _ => 0,
                    };
                    let mut d = depth;
                    let mut k = 0;
                    while k < bytes.len() && d > 0 {
                        match bytes[k] {
                            b'\\' => {
                                k += 2;
                            }
                            c if open != 0 && c == open => {
                                d += 1;
                                k += 1;
                            }
                            c if c == close => {
                                d -= 1;
                                k += 1;
                            }
                            _ => {
                                k += 1;
                            }
                        }
                    }
                    if d == 0 { None } else { Some((close, d)) }
                };
                in_multiline_percent = new_state;
                i += 1;
                continue;
            }

            // ── Pure comment lines ───────────────────────────────────────────
            if line.trim_start().starts_with('#') {
                i += 1;
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let line_bytes = line.as_bytes();
            let mut in_string: Option<u8> = None;
            let mut in_regex = in_multiline_regex;
            let mut j = 0;

            while j < line_bytes.len() {
                let b = line_bytes[j];

                // ── Inside /regex/ ───────────────────────────────────────────
                if in_regex {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        b'[' => {
                            // Character class — skip until `]`
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
                        b'/' => {
                            in_regex = false;
                            j += 1;
                            continue;
                        }
                        _ => {
                            j += 1;
                            continue;
                        }
                    }
                }

                // ── Inside a string ──────────────────────────────────────────
                match in_string {
                    Some(_) if b == b'\\' => {
                        j += 2;
                        continue;
                    }
                    // String interpolation #{...}
                    Some(b'"')
                        if b == b'#'
                            && j + 1 < line_bytes.len()
                            && line_bytes[j + 1] == b'{' =>
                    {
                        j += 2; // skip `#{`
                        let mut depth = 1usize;
                        while j < line_bytes.len() && depth > 0 {
                            let ib = line_bytes[j];
                            if ib == b'\\' {
                                j += 2;
                                continue;
                            }
                            if ib == b'{' {
                                depth += 1;
                                j += 1;
                                continue;
                            }
                            if ib == b'}' {
                                depth -= 1;
                                if depth == 0 {
                                    j += 1;
                                    break;
                                }
                                j += 1;
                                continue;
                            }
                            // Nested string inside interpolation
                            if ib == b'"' || ib == b'\'' || ib == b'`' {
                                let id = ib;
                                j += 1;
                                while j < line_bytes.len() {
                                    if line_bytes[j] == b'\\' {
                                        j += 2;
                                        continue;
                                    }
                                    if line_bytes[j] == id {
                                        j += 1;
                                        break;
                                    }
                                    j += 1;
                                }
                                continue;
                            }
                            j += 1;
                        }
                        continue;
                    }
                    Some(delim) if b == delim => {
                        in_string = None;
                    }
                    Some(_) => {}

                    // ── Outside strings/regex ────────────────────────────────
                    None if b == b'"' || b == b'\'' || b == b'`' => {
                        in_string = Some(b);
                    }
                    None if b == b'#' => break, // inline comment — stop scanning

                    // Percent literals: %r, %q, %Q, %w, %W, %i, %I, %s, %x,
                    // and bare % followed by any punctuation delimiter.
                    None if b == b'%' && j + 1 < line_bytes.len() => {
                        let (new_j, ml_state) = skip_percent_literal(line_bytes, j);
                        if let Some(state) = ml_state {
                            in_multiline_percent = Some(state);
                        }
                        j = new_j;
                        continue;
                    }

                    // /regex/ literal
                    None if b == b'/' && slash_starts_regex(line_bytes, j) => {
                        in_regex = true;
                        j += 1;
                        continue;
                    }

                    // `$,` — Ruby field-separator global variable; the `,` is
                    // part of the variable name, not a comma separator.
                    None if b == b'$'
                        && j + 1 < line_bytes.len()
                        && line_bytes[j + 1] == b',' =>
                    {
                        j += 2; // skip `$,`
                        continue;
                    }

                    None if b == b',' => {
                        let next = line_bytes.get(j + 1).copied();
                        // Don't flag trailing commas before closing brackets/parens/braces
                        // (e.g. `[a, b,]` or `foo(a, b,)`) — rubocop allows these.
                        let is_trailing = matches!(next, Some(b']') | Some(b')') | Some(b'}') | Some(b'>'));
                        if !is_trailing && next != Some(b' ') && next != Some(b'\t') && next.is_some() {
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

            in_multiline_regex = in_regex;

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
