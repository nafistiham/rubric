use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SafeNavigationWithEmpty;

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

impl Rule for SafeNavigationWithEmpty {
    fn name(&self) -> &'static str {
        "Lint/SafeNavigationWithEmpty"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut heredoc_term: Option<String> = None;
        let pattern = b"&.empty?";

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

            let mut search = 0usize;
            while search < bytes.len() {
                if let Some(rel) = bytes[search..]
                    .windows(pattern.len())
                    .position(|w| w == pattern)
                {
                    let abs = search + rel;

                    // Check that after `&.empty?` the next char is not `(` (would be a method call
                    // on a different receiver) — actually `&.empty?` is always the pattern we want.
                    // Ensure the character after the match is not alphanumeric (word boundary).
                    let after = abs + pattern.len();
                    let after_ok = after >= bytes.len()
                        || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');

                    if after_ok && !in_string_at(bytes, abs) {
                        let start = (line_start + abs) as u32;
                        let end = start + pattern.len() as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Do not use safe navigation operator with empty?.".into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }
                    search = abs + pattern.len();
                } else {
                    break;
                }
            }
        }

        diags
    }
}
