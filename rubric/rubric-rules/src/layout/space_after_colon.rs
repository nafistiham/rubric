use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAfterColon;

/// If `line` opens a heredoc, return the terminator string (e.g., "EOM", "RUBY").
/// Handles `<<WORD`, `<<-WORD`, `<<~WORD`, and quoted variants `<<~'WORD'`.
fn detect_heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i + 1 < n {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            i += 2;
            if i < n && (bytes[i] == b'-' || bytes[i] == b'~') {
                i += 1;
            }
            // Optional surrounding quote (<<~'EOM', <<~"EOM")
            if i < n && (bytes[i] == b'\'' || bytes[i] == b'"' || bytes[i] == b'`') {
                i += 1;
            }
            let word_start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            if i > word_start {
                return Some(line[word_start..i].to_string());
            }
        } else {
            i += 1;
        }
    }
    None
}

/// Skip `#{...}` interpolation in `bytes` starting at `j` (which points at
/// `#`; caller has already verified `bytes[j+1] == b'{'`).
/// Returns the position after the closing `}`.
fn skip_interpolation(bytes: &[u8], start: usize) -> usize {
    let n = bytes.len();
    let mut j = start + 2; // skip `#{`
    let mut depth = 1usize;
    while j < n && depth > 0 {
        match bytes[j] {
            b'\\' => {
                j += 2;
            }
            b'{' => {
                depth += 1;
                j += 1;
            }
            b'}' => {
                depth -= 1;
                j += 1;
            }
            // Nested string inside interpolation — skip its content
            b'"' | b'\'' | b'`' => {
                let delim = bytes[j];
                j += 1;
                while j < n {
                    if bytes[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if bytes[j] == delim {
                        j += 1;
                        break;
                    }
                    j += 1;
                }
            }
            _ => {
                j += 1;
            }
        }
    }
    j
}

impl Rule for SpaceAfterColon {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterColon"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        // Cross-line heredoc tracking
        let mut in_heredoc: Option<String> = None;
        // Cross-line regex tracking (multiline /regex/x)
        let mut in_multiline_regex = false;
        // Cross-line string tracking (multiline backtick strings)
        // Stores the string delimiter (`'``)
        let mut in_multiline_string: Option<u8> = None;

        for (i, line) in lines.iter().enumerate() {
            // ── Heredoc body ─────────────────────────────────────────────────
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect if this line opens a heredoc (body starts on the next line).
            if let Some(term) = detect_heredoc_terminator(line) {
                in_heredoc = Some(term);
            }

            let bytes = line.as_bytes();
            let len = bytes.len();

            // ── Continuing a multiline string (backtick) ─────────────────────
            if let Some(delim) = in_multiline_string {
                // Scan for the closing delimiter
                let mut j = 0;
                let mut closed = false;
                while j < len {
                    match bytes[j] {
                        b'\\' => { j += 2; }
                        c if c == delim => { closed = true; break; }
                        _ => { j += 1; }
                    }
                }
                if closed {
                    in_multiline_string = None;
                    // Fall through: resume normal scanning after the closing delim
                    // (The rest of the line is handled below, but for simplicity
                    // we skip the whole line since the remainder is rare.)
                }
                continue;
            }

            // ── Continuing a multiline /regex/ ───────────────────────────────
            if in_multiline_regex {
                // Scan for the closing `/`
                let mut j = 0;
                while j < len {
                    match bytes[j] {
                        b'\\' => { j += 2; }
                        b'/' => { in_multiline_regex = false; break; }
                        _ => { j += 1; }
                    }
                }
                continue;
            }

            let mut in_string: Option<u8> = None;
            let mut in_regex = false;
            let mut j = 0;

            while j < len {
                let b = bytes[j];

                // ── Skip percent literals: %r{}, %r!!, %w[], %x[], %q(), etc. ──
                if in_string.is_none() && !in_regex && b == b'%' && j + 1 < len {
                    let next_b = bytes[j + 1];
                    let delim_start = match next_b {
                        b'r' | b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' => j + 2,
                        b'(' | b'[' | b'{' | b'|' | b'/' => j + 1,
                        _ => usize::MAX,
                    };
                    if delim_start < len {
                        let open = bytes[delim_start];
                        let close = match open {
                            b'(' => b')',
                            b'[' => b']',
                            b'{' => b'}',
                            b'<' => b'>',
                            _ => open, // symmetric delimiter (!, |, /, etc.)
                        };
                        j = delim_start + 1;
                        if open == close {
                            // Symmetric delimiter: scan until unescaped close
                            while j < len && bytes[j] != close {
                                if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                            }
                            if j < len { j += 1; }
                        } else {
                            // Bracket delimiter: track nesting depth
                            let mut depth = 1usize;
                            while j < len && depth > 0 {
                                match bytes[j] {
                                    b'\\' => { j += 2; }
                                    c if c == open => { depth += 1; j += 1; }
                                    c if c == close => { depth -= 1; j += 1; }
                                    _ => { j += 1; }
                                }
                            }
                        }
                        continue;
                    }
                }

                // ── Regex state: skip until unescaped `/` ────────────────────
                if in_regex {
                    match b {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_regex = false; }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                // ── String state ─────────────────────────────────────────────
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    // `#{...}` interpolation inside double-quoted or backtick string:
                    // skip the interpolation block so the inner `"` doesn't close us.
                    Some(b'"') | Some(b'`')
                        if b == b'#'
                            && j + 1 < len
                            && bytes[j + 1] == b'{' =>
                    {
                        j = skip_interpolation(bytes, j);
                        continue;
                    }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    // Backtick string — can span multiple lines
                    None if b == b'`' => {
                        j += 1;
                        // Scan for closing backtick on this line
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'#' if j + 1 < len && bytes[j + 1] == b'{' => {
                                    j = skip_interpolation(bytes, j);
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
                    None if b == b'#' => break, // inline comment — stop scanning
                    None => {}
                }

                // ── Regex start detection: `/` after operator/open-paren/space ──
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b',' || prev == b'['
                        || prev == b' ' || prev == b'\t' || prev == 0
                        || prev == b'!'
                        || prev == b'?'
                    {
                        in_regex = true;
                        j += 1;
                        // If regex is not closed on this line, mark as multiline
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

                if b == b':' {
                    // Skip `::` (double colon)
                    if j + 1 < len && bytes[j + 1] == b':' {
                        j += 2;
                        continue;
                    }
                    // Skip `:` at end of line
                    if j + 1 >= len {
                        j += 1;
                        continue;
                    }
                    let next = bytes[j + 1];
                    // Skip `:` followed by `]` — POSIX character class / array access
                    if next == b']' {
                        j += 1;
                        continue;
                    }
                    // Skip keyword argument shorthand and required keyword args:
                    // `name:,` `cursor:)` `code:}` `param:|` — no space needed.
                    if next == b',' || next == b')' || next == b'}' || next == b'|' {
                        j += 1;
                        continue;
                    }
                    // Skip URL scheme `://`
                    if next == b'/' && j + 2 < len && bytes[j + 2] == b'/' {
                        j += 1;
                        continue;
                    }
                    // Flag `:` not followed by space when preceded by a word char
                    if next != b' ' && next != b'\n' && next != b'\r' {
                        let preceded_by_word = j > 0
                            && (bytes[j - 1].is_ascii_alphanumeric()
                                || bytes[j - 1] == b'_');
                        if preceded_by_word {
                            let line_start = ctx.line_start_offsets[i];
                            let colon_pos = line_start + j as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Missing space after colon.".into(),
                                range: TextRange::new(colon_pos, colon_pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
                j += 1;
            }
        }

        diags
    }
}
