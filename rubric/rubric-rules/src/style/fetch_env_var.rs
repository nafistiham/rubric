use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FetchEnvVar;

/// Returns the index of the comment character `#` on the line, ignoring
/// `#` that appear inside string literals or string interpolations.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut interp_depth: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                interp_depth += 1;
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d && interp_depth == 0 => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        if interp_depth > 0 && bytes[i] == b'}' {
            interp_depth -= 1;
        }
        i += 1;
    }
    None
}

/// Returns true if the byte position `pos` in `bytes` is inside a string literal.
fn in_string_at(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => {
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// If `line` contains a heredoc opener, returns the terminator string.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    let pos = line.find("<<")?;
    let rest = &line[pos + 2..];
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    let rest = rest.strip_prefix('~').unwrap_or(rest);
    let rest = if rest.starts_with('"') || rest.starts_with('\'') || rest.starts_with('`') {
        &rest[1..]
    } else {
        rest
    };
    let word: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if word.is_empty() { None } else { Some(word) }
}

/// Searches for `ENV[` followed by a quoted string and `]` in `bytes`.
/// Returns a list of byte positions (relative to `bytes`) where each match starts.
/// Skips matches that are inside string literals or prefixed by `.fetch`.
fn find_env_bracket_accesses(bytes: &[u8]) -> Vec<usize> {
    let mut results = Vec::new();
    let needle = b"ENV[";
    let mut search = 0usize;

    while search < bytes.len() {
        let found = bytes[search..]
            .windows(needle.len())
            .position(|w| w == needle);
        let rel = match found {
            Some(r) => r,
            None => break,
        };
        let abs = search + rel;

        // Ensure `ENV` is not prefixed by a word character (e.g. `MYENV[`)
        let preceded_by_word = abs > 0 && {
            let b = bytes[abs - 1];
            b.is_ascii_alphanumeric() || b == b'_'
        };

        if !preceded_by_word && !in_string_at(bytes, abs) {
            // Check that the character after `ENV[` is a quote
            let bracket_pos = abs + 4; // position of `[`
            let after_bracket = abs + 4; // first char inside bracket
            if after_bracket < bytes.len()
                && (bytes[after_bracket] == b'\'' || bytes[after_bracket] == b'"')
            {
                let quote = bytes[after_bracket];
                // Find closing quote
                let mut j = after_bracket + 1;
                while j < bytes.len() && bytes[j] != quote {
                    if bytes[j] == b'\\' {
                        j += 1; // skip escape
                    }
                    j += 1;
                }
                // j now points to closing quote (or end of bytes)
                if j < bytes.len() && bytes[j] == quote {
                    // Expect `]` right after closing quote
                    let close_bracket = j + 1;
                    if close_bracket < bytes.len() && bytes[close_bracket] == b']' {
                        // Check that this is NOT `.fetch` before `ENV`
                        // i.e. no `.fetch` call — the pattern `ENV[` itself is the violation
                        // (ENV.fetch would not match `ENV[`)
                        results.push(abs);
                        // Advance past the entire match
                        search = close_bracket + 1;
                        continue;
                    }
                }
            }
            // No valid match at this position; skip past `ENV[`
            let _ = bracket_pos;
        }
        search = abs + needle.len();
    }

    results
}

impl Rule for FetchEnvVar {
    fn name(&self) -> &'static str {
        "Style/FetchEnvVar"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut heredoc_term: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines
            if let Some(ref term) = heredoc_term {
                if line.trim() == term.as_str() {
                    heredoc_term = None;
                }
                continue;
            }

            if let Some(term) = extract_heredoc_terminator(line) {
                heredoc_term = Some(term);
            }

            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];
            let bytes = scan_slice.as_bytes();

            let line_start = ctx.line_start_offsets[i] as usize;

            for abs in find_env_bracket_accesses(bytes) {
                let start = (line_start + abs) as u32;
                // Highlight just `ENV[`
                let end = start + 4;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use ENV.fetch(key) or ENV.fetch(key, nil) instead of ENV[key].".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
