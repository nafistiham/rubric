use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceBeforeComment;

fn heredoc_terminator(line: &str) -> Option<String> {
    let marker = if let Some(p) = line.find("<<~") {
        Some(&line[p + 3..])
    } else if let Some(p) = line.find("<<-") {
        Some(&line[p + 3..])
    } else if let Some(p) = line.find("<<") {
        let after = &line[p + 2..];
        if after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '"' || c == '\'') {
            Some(after)
        } else { None }
    } else { None }?;
    let rest = marker.trim_start_matches(|c: char| c == '\'' || c == '"');
    let term: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
    if term.is_empty() { None } else { Some(term) }
}

/// Returns the byte offset of the first inline comment `#` on `bytes`, or `None`.
///
/// An "inline comment" is a `#` that:
///   1. Is NOT inside a string literal (`"..."`, `'...'`)
///   2. Is NOT inside a percent literal (`%r{...}`, `%w(...)`, etc.)
///   3. Is NOT inside a `/regex/` literal
///   4. Is NOT a `#{...}` string interpolation opener
///   5. IS followed by a space or end of line (distinguishes `# comment` from
///      bare `#` in contexts like `Array#length` documentation, `#!` shebangs, etc.)
fn find_inline_comment(bytes: &[u8]) -> Option<usize> {
    let n = bytes.len();
    let mut i = 0;
    let mut in_string: Option<u8> = None;
    let mut in_regex = false;
    // Percent-literal state: (close_byte, depth).
    // For brace-style delimiters ({, (, [, <) depth is tracked.
    // For same-char delimiters (|, !, /, ...) depth is irrelevant; any occurrence of
    // close_byte ends the literal.
    let mut pct_state: Option<(u8, u32)> = None;

    while i < n {
        let b = bytes[i];

        // ── Inside a string literal ────────────────────────────────────────
        if let Some(q) = in_string {
            if b == b'\\' && i + 1 < n {
                i += 2; // skip escaped character
                continue;
            }
            if b == b'#' && i + 1 < n && bytes[i + 1] == b'{' {
                // `#{` string interpolation: advance past it; content is handled
                // by the outer-loop state since we remain conceptually in the string.
                i += 2;
                continue;
            }
            if b == q {
                in_string = None;
            }
            i += 1;
            continue;
        }

        // ── Inside a /regex/ literal ───────────────────────────────────────
        if in_regex {
            if b == b'\\' && i + 1 < n {
                i += 2;
                continue;
            }
            if b == b'/' {
                in_regex = false;
            }
            i += 1;
            continue;
        }

        // ── Inside a percent literal ───────────────────────────────────────
        if let Some((close, depth)) = pct_state {
            let open_byte = match close {
                b'}' => b'{',
                b')' => b'(',
                b']' => b'[',
                b'>' => b'<',
                _ => 0, // same-char delimiter
            };
            if open_byte != 0 && b == open_byte {
                pct_state = Some((close, depth + 1));
            } else if b == close {
                if depth <= 1 {
                    pct_state = None;
                } else {
                    pct_state = Some((close, depth - 1));
                }
            }
            i += 1;
            continue;
        }

        // ── Outside any literal ────────────────────────────────────────────
        match b {
            b'"' | b'\'' => {
                in_string = Some(b);
                i += 1;
            }
            b'/' => {
                // Treat as regex opener when preceded by `=`, `(`, `,`, `[`, `|`,
                // space/tab, or at the start of non-whitespace content.
                let prev = if i > 0 { bytes[i - 1] } else { 0 };
                if matches!(prev, b'=' | b'(' | b',' | b'[' | b'|' | b' ' | b'\t' | 0) {
                    in_regex = true;
                }
                i += 1;
            }
            b'%' if i + 1 < n => {
                // Detect percent literals (%r, %q, %Q, %w, %W, %i, %I, %s, %x, %()).
                let mut k = i + 1;
                if k < n && bytes[k].is_ascii_alphabetic() {
                    k += 1; // optional type letter
                }
                if k < n {
                    let delim = bytes[k];
                    let close = match delim {
                        b'{' => b'}',
                        b'(' => b')',
                        b'[' => b']',
                        b'<' => b'>',
                        _ => delim, // same-char delimiter
                    };
                    let depth = if close != delim { 1 } else { 0 };
                    pct_state = Some((close, depth));
                    i = k + 1;
                } else {
                    i += 1;
                }
            }
            b'#' => {
                // Only treat as a comment if followed by a space or end of line.
                // This excludes `Array#length`, `#!` shebangs, `#{}` (handled above), etc.
                let next = bytes.get(i + 1).copied();
                if next == Some(b' ') || next.is_none() || next == Some(b'\t') {
                    // Backtrack to the first `#` in a consecutive `###...` run so that
                    // `check_source` checks spacing before the whole comment token, not
                    // just the last `#` in a `##` / `###` sequence.
                    let mut start = i;
                    while start > 0 && bytes[start - 1] == b'#' {
                        start -= 1;
                    }
                    return Some(start);
                }
                // Not a comment marker — continue scanning.
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    None
}

impl Rule for SpaceBeforeComment {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeComment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut in_heredoc = false;
        let mut heredoc_term = String::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip heredoc body lines — their content is string data, not code.
            if in_heredoc {
                if line.trim() == heredoc_term.as_str() {
                    in_heredoc = false;
                    heredoc_term.clear();
                }
                continue;
            }

            // Skip standalone comment lines — only flag inline comments after code.
            if line.trim_start().starts_with('#') {
                continue;
            }
            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            if let Some(j) = find_inline_comment(bytes) {
                if j == 0 {
                    continue; // Whole line is a comment (already skipped, but be safe).
                }
                let prev = bytes[j - 1];
                if prev != b' ' && prev != b'\t' {
                    let pos = (line_start + j) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Put a space before an inline comment.".into(),
                        range: TextRange::new(pos, pos + 1),
                        severity: Severity::Warning,
                    });
                }
            }

            if let Some(term) = heredoc_terminator(line) {
                in_heredoc = true;
                heredoc_term = term;
            }
        }
        diags
    }
}
