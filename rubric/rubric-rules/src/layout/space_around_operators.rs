use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAroundOperators;

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

impl Rule for SpaceAroundOperators {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundOperators"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Cross-line heredoc tracking: Some(terminator) while inside a heredoc body.
        let mut in_heredoc: Option<String> = None;
        // Cross-line percent-word-array tracking: true while inside %w[...], %W[...],
        // %i[...], %I[...] that spans multiple lines. Lines inside are plain word tokens —
        // no operator scanning applies.
        let mut in_percent_word_array = false;
        // Cross-line multiline /regex/ tracking: true when a /regex/ started on a
        // previous line and hasn't been closed yet.
        let mut in_multiline_regex = false;
        // Cross-line multiline %r{...} tracking: Some((close_byte, depth)) when a
        // %r percent-regex literal spans multiple lines.
        let mut in_multiline_percent_regex: Option<(u8, usize)> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip lines that are inside a heredoc body (including the terminator line).
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }

            // Skip lines inside a multiline /regex/.  Scan for the closing `/` so we
            // know when the regex ends; don't flag operators inside the body.
            if in_multiline_regex {
                let bytes = line.as_bytes();
                let mut k = 0;
                while k < bytes.len() {
                    match bytes[k] {
                        b'\\' => { k += 2; }
                        b'/' => { in_multiline_regex = false; k += 1; break; }
                        _ => { k += 1; }
                    }
                }
                continue;
            }

            // Skip lines inside a multiline %r{...} literal.
            if let Some((close, ref mut depth)) = in_multiline_percent_regex {
                let open = match close { b')' => b'(', b']' => b'[', b'}' => b'{', b'>' => b'<', c => c };
                let bytes = line.as_bytes();
                let mut k = 0;
                while k < bytes.len() {
                    match bytes[k] {
                        b'\\' => { k += 2; }
                        c if c == open => { *depth += 1; k += 1; }
                        c if c == close => {
                            *depth -= 1;
                            k += 1;
                            if *depth == 0 { break; }
                        }
                        _ => { k += 1; }
                    }
                }
                if *depth == 0 {
                    in_multiline_percent_regex = None;
                }
                continue;
            }

            // Skip =begin / =end embedded documentation delimiters.
            // These always appear at column 0 and are NOT assignment operators.
            let trimmed_line = line.trim_start();
            if trimmed_line == "=begin" || trimmed_line.starts_with("=begin ")
                || trimmed_line.starts_with("=begin\t")
            {
                continue;
            }
            if trimmed_line == "=end" || trimmed_line.starts_with("=end ")
                || trimmed_line.starts_with("=end\t")
            {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;
            let mut in_string: Option<u8> = None;
            let mut in_regex = false;
            // Track if this line is a def header so we can skip = in param defaults
            let is_def_line = trimmed_line.starts_with("def ");
            let mut paren_depth: i32 = 0;

            // If we are inside a multiline %w/%W/%i/%I array, scan for the closing `]`
            // to end the context, then skip operator checking for this line.
            if in_percent_word_array {
                if line.contains(']') {
                    in_percent_word_array = false;
                }
                continue;
            }

            // Detect if this line opens a heredoc (body starts on the next line).
            if let Some(term) = detect_heredoc_terminator(line) {
                in_heredoc = Some(term);
                // Fall through: still check operators on the opener line itself.
            }

            while j < len {
                let b = bytes[j];

                // ── Skip percent literals: %r{}, %w[], %i[], %(str), %q(), %Q() ──
                // Also handles backtick strings as a special case below.
                if b == b'%' && j + 1 < len {
                    let next_b = bytes[j+1];
                    // Determine delimiter: skip optional type char (r, w, W, i, I, q, Q, x)
                    let (type_skip, delim_pos) = match next_b {
                        b'r' | b'w' | b'W' | b'i' | b'I' | b'q' | b'Q' | b'x' => (true, j + 2),
                        b'(' | b'[' | b'{' | b'|' | b'/' => (false, j + 1),
                        _ => (false, usize::MAX), // not a percent literal
                    };
                    if delim_pos < len || (!type_skip && delim_pos == usize::MAX) {
                        let delim_start = if type_skip { j + 2 } else { j + 1 };
                        if delim_start < len {
                            let open = bytes[delim_start];
                            let close = match open {
                                b'(' => b')',
                                b'[' => b']',
                                b'{' => b'}',
                                b'<' => b'>',
                                _ => open, // symmetric delimiter
                            };
                            j = delim_start + 1;
                            if open == close {
                                // Symmetric delimiter: scan until unescaped close
                                while j < len && bytes[j] != close {
                                    if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                                }
                                if j < len { j += 1; }
                            } else {
                                // Bracket delimiter: track depth
                                let mut depth = 1usize;
                                while j < len && depth > 0 {
                                    match bytes[j] {
                                        b'\\' => { j += 2; }
                                        c if c == open => { depth += 1; j += 1; }
                                        c if c == close => { depth -= 1; j += 1; }
                                        _ => { j += 1; }
                                    }
                                }
                                // If we reached end of line without closing:
                                // - %w/%W/%i/%I → word array (no operator scanning)
                                // - %r → regex literal (no operator scanning)
                                if depth > 0 {
                                    if matches!(next_b, b'w' | b'W' | b'i' | b'I') {
                                        in_percent_word_array = true;
                                    } else if next_b == b'r' {
                                        in_multiline_percent_regex = Some((close, depth));
                                    }
                                    break;
                                }
                            }
                            continue;
                        }
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
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' || b == b'`' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment — stop scanning
                    None => {}
                }

                // ── 3-char compound operators: ||=, &&=, ===, <=> ────────────
                if j + 2 < len {
                    let three = &bytes[j..j+3];
                    if three == b"||=" || three == b"&&=" || three == b"===" || three == b"<=>" {
                        // Skip if in symbol context: :<=>
                        if j > 0 && bytes[j-1] == b':' {
                            j += 3;
                            continue;
                        }
                        let prev_ok = j == 0 || bytes[j-1] == b' ' || bytes[j-1] == b'\t';
                        let next_ok = j + 3 >= len || bytes[j+3] == b' ' || bytes[j+3] == b'\t';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Operator `{}` should be surrounded by spaces.",
                                    std::str::from_utf8(three).unwrap_or("?")
                                ),
                                range: TextRange::new(pos, pos + 3),
                                severity: Severity::Warning,
                            });
                        }
                        j += 3;
                        continue;
                    }
                }

                // ── 2-char operators ─────────────────────────────────────────
                if j + 1 < len {
                    let two = &bytes[j..j+2];

                    // ** — exponentiation OR double-splat.
                    // It's a double-splat (**opts / **hash) when:
                    //   - the next char is an identifier char, AND
                    //   - the preceding char is NOT an identifier/digit/closing bracket
                    //     (i.e. we're after `(`, `,`, space, `{`, etc.)
                    if two == b"**" {
                        let next_b = if j + 2 < len { bytes[j+2] } else { 0 };
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if (next_b.is_ascii_alphabetic() || next_b == b'_')
                            && !prev.is_ascii_alphanumeric() && prev != b')' && prev != b']'
                        {
                            // double-splat: **opts / **hash — no spacing required
                            j += 2;
                            continue;
                        }
                        // exponentiation — no spacing required (RuboCop default: no_space style)
                        j += 2;
                        continue;
                    }

                    // Skip ->
                    if two == b"->" {
                        j += 2;
                        continue;
                    }

                    // Remaining two-char operators: ==, !=, <=, >=, &&, ||, +=, -=, *=, /=
                    if two == b"==" || two == b"!=" || two == b"<=" || two == b">="
                        || two == b"&&" || two == b"||" || two == b"+=" || two == b"-="
                        || two == b"*=" || two == b"/="
                    {
                        // Skip <<= (left-shift-assign): when we see `<=` preceded by `<`
                        // Skip >>= (right-shift-assign): when we see `>=` preceded by `>`
                        if (two == b"<=" && j > 0 && bytes[j-1] == b'<')
                            || (two == b">=" && j > 0 && bytes[j-1] == b'>')
                        {
                            j += 2;
                            continue;
                        }
                        // Skip operator symbols: :<=, :>=, :==, :!=, :&&, :||
                        if j > 0 && bytes[j-1] == b':' {
                            j += 2;
                            continue;
                        }
                        let prev_ok = j == 0 || bytes[j-1] == b' ' || bytes[j-1] == b'\t';
                        let next_ok = j + 2 >= len || bytes[j+2] == b' ' || bytes[j+2] == b'\t' || bytes[j+2] == b'\n';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Operator `{}` should be surrounded by spaces.",
                                    std::str::from_utf8(two).unwrap_or("?")
                                ),
                                range: TextRange::new(pos, pos + 2),
                                severity: Severity::Warning,
                            });
                        }
                        j += 2;
                        continue;
                    }
                }

                // ── Single-char operators: = + - * / < > ────────────────────
                match b {
                    b'=' => {
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        // Skip trailing char of !=, <=, >=, ==
                        // Also skip compound assignment operators: |=, &=, ^=
                        if prev == b'!' || prev == b'<' || prev == b'>' || prev == b'='
                            || prev == b'|' || prev == b'&' || prev == b'^'
                        {
                            j += 1;
                            continue;
                        }
                        // Skip => (hash rocket)
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        if next == b'>' {
                            j += 1;
                            continue;
                        }
                        // Skip =~ (regex match operator)
                        if next == b'~' {
                            j += 1;
                            continue;
                        }
                        // Skip setter method definitions: def foo=(val)
                        // The `=` is part of the method name when followed by `(`
                        if next == b'(' {
                            j += 1;
                            continue;
                        }
                        // Skip `[]=` operator as symbol: :[], :[]=
                        if prev == b']' {
                            j += 1;
                            continue;
                        }
                        // Scan backward through identifier chars; check predecessor
                        {
                            let mut k = j;
                            while k > 0 && (bytes[k-1].is_ascii_alphanumeric() || bytes[k-1] == b'_') {
                                k -= 1;
                            }
                            // Skip setter method calls: self.foo=val, obj.attr=val
                            if k > 0 && bytes[k-1] == b'.' {
                                j += 1;
                                continue;
                            }
                            // Skip symbol method name: :config=, :bid=, :name=
                            if k > 0 && bytes[k-1] == b':' {
                                j += 1;
                                continue;
                            }
                        }
                        // Skip = in method parameter defaults: def method(arg=default)
                        if is_def_line && paren_depth > 0 {
                            j += 1;
                            continue;
                        }
                        let prev_ok = j == 0 || prev == b' ' || prev == b'\t';
                        let next_ok = next == b' ' || next == b'\t' || next == 0;
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Operator `=` should be surrounded by spaces.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    b'/' => {
                        // `/` after an operator/open-paren/space/unary-! is a regex start.
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if prev == b'=' || prev == b'(' || prev == b',' || prev == b'['
                            || prev == b' ' || prev == b'\t' || prev == 0
                            || prev == b'!'  // !/regex/ is valid Ruby
                        {
                            in_regex = true;
                            j += 1;
                            continue;
                        }
                        // Skip `?/` — Ruby character literal for the `/` character.
                        // The `?` immediately precedes the slash; there is no division here.
                        if prev == b'?' {
                            j += 1;
                            continue;
                        }
                        // Otherwise treat as division and check spacing.
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        // Skip if it's /= (already handled in two-char section, but defensive)
                        if next == b'=' {
                            j += 1;
                            continue;
                        }
                        let prev_ok = prev == b' ' || prev == b'\t';
                        let next_ok = next == b' ' || next == b'\t';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Operator `/` should be surrounded by spaces.".into(),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    b'+' | b'-' | b'*' => {
                        // Skip if next is = (+=, -=, *=) — handled in two-char section
                        if j + 1 < len && bytes[j+1] == b'=' {
                            j += 1;
                            continue;
                        }
                        // Skip * when next is * — already handled as ** above
                        if b == b'*' && j + 1 < len && bytes[j+1] == b'*' {
                            j += 1;
                            continue;
                        }
                        // Skip unary: operator at start of line or after (, [, ,, space, =, operator
                        let prev = if j > 0 { bytes[j-1] } else { 0 };
                        if prev == b'(' || prev == b'[' || prev == b',' || prev == b' '
                            || prev == b'\t' || prev == 0 || prev == b'=' || prev == b'+'
                            || prev == b'-' || prev == b'*' || prev == b'/'
                            // Splat/unary after `|` in block params: |*args|
                            || prev == b'|'
                            // `-` in `<<-RUBY` heredoc sigil
                            || prev == b'<'
                        {
                            j += 1;
                            continue;
                        }
                        // Skip symbol literals: `:+`, `:-`, `:*` (e.g., `reduce(:+)`)
                        if prev == b':' {
                            j += 1;
                            continue;
                        }
                        // Skip negative/positive number literals
                        let next = if j + 1 < len { bytes[j+1] } else { 0 };
                        if (b == b'-' || b == b'+') && next.is_ascii_digit() && prev != b' ' {
                            j += 1;
                            continue;
                        }
                        let prev_ok = prev == b' ' || prev == b'\t';
                        let next_ok = next == b' ' || next == b'\t';
                        if !prev_ok || !next_ok {
                            let pos = (line_start + j) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!("Operator `{}` should be surrounded by spaces.", b as char),
                                range: TextRange::new(pos, pos + 1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    _ => {
                        // Track paren depth to detect def parameter lists
                        if b == b'(' { paren_depth += 1; }
                        else if b == b')' { paren_depth -= 1; }
                    }
                }

                j += 1;
            }

            // If we finished the line while still inside a /regex/ literal, the regex
            // spans multiple lines — mark it so we skip the continuation lines.
            if in_regex {
                in_multiline_regex = true;
            }
        }

        diags
    }
}
