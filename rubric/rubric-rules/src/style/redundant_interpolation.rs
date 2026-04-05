use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantInterpolation;

/// Returns true if the token starting at `start` in `bytes` is a double-quoted
/// string whose only content is a single `#{...}` interpolation — i.e., the
/// string matches exactly `"#{...}"` with nothing else inside.
///
/// Returns `Some(end_index)` (exclusive, pointing past the closing `"`) on
/// match, `None` otherwise.
fn is_pure_interpolation(bytes: &[u8], start: usize) -> Option<usize> {
    let n = bytes.len();

    // Must start with `"#{ `
    if start + 3 >= n {
        return None;
    }
    if bytes[start] != b'"' || bytes[start + 1] != b'#' || bytes[start + 2] != b'{' {
        return None;
    }

    // Walk forward from the `{`, tracking brace depth, until we find the
    // matching `}`.  After the `}` the very next byte must be `"` and there
    // must be no other content between the opening `"` and the closing `"`.
    let mut depth: usize = 1;
    let mut i = start + 3; // first byte after `#{`

    while i < n {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    // The character immediately after `}` must be the closing `"`
                    let close_brace = i;
                    let after = close_brace + 1;
                    if after < n && bytes[after] == b'"' {
                        return Some(after + 1);
                    }
                    // There is extra content between `}` and the closing `"` — not pure
                    return None;
                }
            }
            b'\\' => {
                // skip escaped character inside interpolation
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    None
}

impl Rule for RedundantInterpolation {
    fn name(&self) -> &'static str {
        "Style/RedundantInterpolation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Cross-line heredoc tracking: when Some, we're inside a heredoc body and
        // the value is the expected terminator (bare identifier, no <<~/-).
        let mut in_heredoc: Option<String> = None;

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines — `"` chars inside are literal content, not
            // Ruby string delimiters, so interpolation detection would be wrong.
            if let Some(ref term) = in_heredoc {
                if line.trim_end_matches(['\r', '\n']) == term.as_str()
                    || line.trim_end_matches(['\r', '\n']).trim_start() == term.as_str()
                {
                    in_heredoc = None;
                }
                continue;
            }

            let trimmed = line.trim_start();
            // Skip full-line comments
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[line_idx] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut i = 0usize;
            let mut in_single_quote = false;
            let mut in_percent_lit = false;
            let mut percent_close: u8 = 0;
            let mut percent_open: u8 = 0;
            let mut percent_depth: i32 = 0;
            let mut in_regex = false; // inside /regex/ — `"` chars are literal

            while i < n {
                let b = bytes[i];

                // Track percent string literals: %(, %Q(, %w(, %r!, etc.
                // When inside, `"` chars are literal content, not string delimiters.
                if in_percent_lit {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    }
                    if percent_open != percent_close {
                        // Paired delimiter — track depth
                        if b == percent_open { percent_depth += 1; }
                        if b == percent_close {
                            percent_depth -= 1;
                            if percent_depth == 0 { in_percent_lit = false; }
                        }
                    } else if b == percent_close {
                        in_percent_lit = false;
                    }
                    i += 1;
                    continue;
                }

                // Track /regex/ context — `"` inside regex is a literal char
                if in_regex {
                    if b == b'\\' { i += 2; continue; }
                    if b == b'/' { in_regex = false; }
                    i += 1;
                    continue;
                }

                // Track single-quoted strings (no interpolation inside)
                if in_single_quote {
                    match b {
                        b'\\' => i += 1, // skip escaped char
                        b'\'' => in_single_quote = false,
                        _ => {}
                    }
                    i += 1;
                    continue;
                }

                match b {
                    b'#' => break, // inline comment — stop scanning
                    b'/' => {
                        // Detect regex opener: preceded by =, (, ,, [, !, |, &, ?, :, ;, {
                        let prev_nonws = bytes[..i].iter().rposition(|&c| c != b' ' && c != b'\t')
                            .map(|p| bytes[p]);
                        if matches!(prev_nonws, None
                            | Some(b'=') | Some(b'(') | Some(b',') | Some(b'[')
                            | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?')
                            | Some(b':') | Some(b';') | Some(b'{') | Some(b'>')) {
                            in_regex = true;
                            i += 1;
                            continue;
                        }
                        i += 1;
                    }
                    b'%' if i + 1 < n => {
                        // Detect percent literals: %(, %Q(, %q(, %w(, %W(, %r!, etc.
                        let next = bytes[i + 1];
                        let (open, advance) =
                            if matches!(next, b'Q' | b'q' | b'W' | b'w' | b'I' | b'i' | b'r' | b'x' | b's') {
                                (bytes.get(i + 2).copied().unwrap_or(0), 3usize)
                            } else if matches!(next, b'(' | b'[' | b'{' | b'<' | b'!' | b'|' | b'/' | b'@' | b'`') {
                                (next, 2usize)
                            } else {
                                (0, 1usize)
                            };
                        if open != 0 {
                            let close = match open {
                                b'(' => b')',
                                b'[' => b']',
                                b'{' => b'}',
                                b'<' => b'>',
                                other => other,
                            };
                            in_percent_lit = true;
                            percent_open = open;
                            percent_close = close;
                            percent_depth = 1;
                            i += advance;
                            continue;
                        }
                        i += 1;
                    }
                    b'\'' => {
                        in_single_quote = true;
                        i += 1;
                    }
                    b'"' => {
                        if let Some(end) = is_pure_interpolation(bytes, i) {
                            // Skip symbol contexts — rubocop does not flag:
                            //   :"#{expr}"   (symbol literal with interpolation)
                            //   "#{expr}":   (dynamic symbol hash key shorthand)
                            let preceded_by_colon = i > 0 && bytes[i - 1] == b':';
                            let followed_by_colon = end < n && bytes[end] == b':';
                            if preceded_by_colon || followed_by_colon {
                                // Skip this string as non-flaggable symbol context
                                i = end;
                            } else {
                            let start_offset = (line_start + i) as u32;
                            let end_offset = (line_start + end) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use `to_s` instead of interpolation.".into(),
                                range: TextRange::new(start_offset, end_offset),
                                severity: Severity::Warning,
                            });
                            i = end;
                            }
                        } else {
                            // Skip past this double-quoted string without flagging
                            i += 1;
                            while i < n {
                                match bytes[i] {
                                    b'\\' => i += 1,
                                    b'"' => {
                                        i += 1;
                                        break;
                                    }
                                    _ => {}
                                }
                                i += 1;
                            }
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            // Detect heredoc opener on this line: <<~TERM, <<-TERM, <<TERM
            // (also handles quoted terminators like <<~'TERM' or <<~"TERM")
            if in_heredoc.is_none() {
                let raw = line;
                let mut search = raw.as_bytes();
                while let Some(pos) = search.windows(2).position(|w| w == b"<<") {
                    let rest = &search[pos + 2..];
                    // Strip optional ~ or -
                    let rest = rest.strip_prefix(b"~").unwrap_or(rest);
                    let rest = rest.strip_prefix(b"-").unwrap_or(rest);
                    // Strip optional quote around terminator
                    let rest = rest.strip_prefix(b"'").unwrap_or_else(|| rest.strip_prefix(b"\"").unwrap_or(rest));
                    // Read identifier chars
                    let term_end = rest.iter().position(|&b| !b.is_ascii_alphanumeric() && b != b'_').unwrap_or(rest.len());
                    if term_end > 0 {
                        let term = std::str::from_utf8(&rest[..term_end]).unwrap_or("").to_string();
                        in_heredoc = Some(term);
                        break;
                    }
                    search = &search[pos + 2..];
                }
            }
        }

        diags
    }
}
