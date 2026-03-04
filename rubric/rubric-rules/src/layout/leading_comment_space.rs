use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct LeadingCommentSpace;

/// Extract the heredoc terminator word from a line that contains `<<` or `<<-` or `<<~`.
/// Returns the bare terminator string (without quotes) if found, or `None`.
fn extract_heredoc_terminator(line: &str) -> Option<String> {
    // Find `<<` in the line — it may appear after other code (e.g., `raise <<~MSG`)
    let mut search = line;
    while let Some(pos) = search.find("<<") {
        let rest = &search[pos + 2..];
        // The character after `<<` may be `-` or `~` (indented heredoc markers)
        let rest = rest.trim_start_matches('-').trim_start_matches('~');
        if rest.is_empty() {
            break;
        }
        // Terminator may be bare (WORD), single-quoted ('WORD'), or double-quoted ("WORD")
        let terminator = if rest.starts_with('\'') {
            let end = rest[1..].find('\'').map(|i| &rest[1..1 + i]);
            end.map(|s| s.to_owned())
        } else if rest.starts_with('"') {
            let end = rest[1..].find('"').map(|i| &rest[1..1 + i]);
            end.map(|s| s.to_owned())
        } else {
            // Bare identifier: letters, digits, underscore
            let end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end > 0 {
                Some(rest[..end].to_owned())
            } else {
                None
            }
        };
        if let Some(t) = terminator {
            if !t.is_empty() {
                return Some(t);
            }
        }
        // Advance past this `<<`
        search = &search[pos + 2..];
    }
    None
}

impl Rule for LeadingCommentSpace {
    fn name(&self) -> &'static str {
        "Layout/LeadingCommentSpace"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // When `Some(terminator)`, we are inside a heredoc and skip lines
        // until we see a line whose trimmed content equals the terminator.
        let mut heredoc_terminator: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            // ── Heredoc tracking ─────────────────────────────────────────────
            if let Some(ref term) = heredoc_terminator.clone() {
                // The terminator line is trimmed (<<~) or exact (<<-)
                let trimmed_line = line.trim();
                if trimmed_line == term.as_str() {
                    heredoc_terminator = None;
                }
                // Either way: this line is inside (or ends) the heredoc — skip it
                continue;
            }

            // Check whether this line opens a heredoc.  We must do this on the
            // opening line *before* the comment check so that lines like
            //   raise <<~MSG
            // are processed for heredoc tracking even though they are not comments.
            if let Some(term) = extract_heredoc_terminator(line) {
                heredoc_terminator = Some(term);
                // The opening line itself is never a comment starting with `#`
                // (it contains code that happens to have `<<`), so fall through
                // to the comment check — it will be skipped anyway because the
                // trimmed content won't start with `#`.
            }

            let trimmed = line.trim_start();
            // Only check lines where the first non-space character is `#`
            if !trimmed.starts_with('#') {
                continue;
            }

            let bytes = trimmed.as_bytes();
            // Just `#` alone — OK
            if bytes.len() == 1 {
                continue;
            }

            // Skip shebangs `#!`
            if bytes[1] == b'!' {
                continue;
            }

            // Skip `##` (YARD doc comment markers, RDoc section headers)
            if bytes[1] == b'#' {
                continue;
            }

            // Skip `#{` — this is Ruby string interpolation syntax, not a
            // comment.  It appears in heredoc bodies, percent-literals, and
            // double-quoted strings.  A line whose first non-space character is
            // `#{` is always interpolation content, never a real comment.
            if bytes[1] == b'{' {
                continue;
            }

            // Skip encoding/magic comments like `# encoding:`, `# frozen_string_literal:`
            let after_hash = &trimmed[1..];
            if after_hash.starts_with(" encoding:")
                || after_hash.starts_with(" frozen_string_literal:")
            {
                continue;
            }

            // Flag if char after `#` is not a space
            if bytes[1] != b' ' {
                // Find offset of `#` in original line
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let hash_pos = (line_start + indent) as u32;

                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Missing space after `#` in comment.".into(),
                    range: TextRange::new(hash_pos, hash_pos + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Replace `#` with `# ` (insert space after hash)
        Some(Fix {
            edits: vec![TextEdit {
                range: TextRange::new(diag.range.start, diag.range.end),
                replacement: "# ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}
