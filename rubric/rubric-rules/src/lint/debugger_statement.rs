use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DebuggerStatement;

/// Debugger entry points to detect, as `(pattern_bytes, display_name)`.
/// Ordered more-specific before less-specific so the `matched_ranges` guard
/// prevents a longer pattern from also triggering a shorter sub-pattern.
static DEBUGGER_PATTERNS: &[(&[u8], &str)] = &[
    (b"binding.remote_pry", "binding.remote_pry"),
    (b"binding.pry_remote", "binding.pry_remote"),
    (b"binding.break",      "binding.break"),
    (b"binding.pry",        "binding.pry"),
    (b"remote_byebug",      "remote_byebug"),
    (b"Pry.start",          "Pry.start"),
    (b"byebug",             "byebug"),
    (b"debugger",           "debugger"),
];

/// Returns true if the byte position `pos` in `bytes` is inside a string literal.
/// Stops at an unquoted `#` (real comment).
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
                // Real comment: nothing after this is code
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Returns true if the character immediately before `pos` in `bytes` is a
/// word boundary (not alphanumeric or `_`).
fn word_boundary_before(bytes: &[u8], pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let ch = bytes[pos - 1];
    !ch.is_ascii_alphanumeric() && ch != b'_'
}

/// Returns true if the character at `end_pos` in `bytes` is a word boundary
/// (not alphanumeric or `_`).
fn word_boundary_after(bytes: &[u8], end_pos: usize) -> bool {
    if end_pos >= bytes.len() {
        return true;
    }
    let ch = bytes[end_pos];
    !ch.is_ascii_alphanumeric() && ch != b'_'
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

impl Rule for DebuggerStatement {
    fn name(&self) -> &'static str {
        "Lint/Debugger"
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

            // Detect heredoc opener before processing line
            if let Some(term) = extract_heredoc_terminator(line) {
                heredoc_term = Some(term);
                // Fall through: opening line is real Ruby code
            }

            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Collect (start_byte, end_byte) of all matches on this line.
            // We use a matched-ranges list so that a longer pattern that was
            // already reported prevents a shorter sub-pattern from double-reporting.
            let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

            for &(pattern, display) in DEBUGGER_PATTERNS {
                let mut search = 0usize;
                while search + pattern.len() <= bytes.len() {
                    let found = bytes[search..].windows(pattern.len())
                        .position(|w| w == pattern);

                    let rel = match found {
                        Some(r) => r,
                        None => break,
                    };

                    let abs = search + rel;
                    let end = abs + pattern.len();

                    // Advance search past this position regardless of whether we flag it
                    search = abs + 1;

                    // Word boundary checks
                    if !word_boundary_before(bytes, abs) || !word_boundary_after(bytes, end) {
                        continue;
                    }

                    // Skip if inside string or comment
                    if in_string_at(bytes, abs) {
                        continue;
                    }

                    // Skip if already covered by a longer pattern's match
                    let already_covered = matched_ranges
                        .iter()
                        .any(|&(ms, me)| abs >= ms && end <= me);
                    if already_covered {
                        continue;
                    }

                    let byte_start = (line_start + abs) as u32;
                    let byte_end = (line_start + end) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Remove debugger entry point `{}`.", display),
                        range: TextRange::new(byte_start, byte_end),
                        severity: Severity::Warning,
                    });
                    matched_ranges.push((abs, end));
                }
            }
        }

        diags
    }
}
