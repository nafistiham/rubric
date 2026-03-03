use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NestedMethodDefinition;

/// Returns `true` when the trimmed line is an endless method definition,
/// i.e. `def name = expr` or `def name(params) = expr`.
///
/// Detection strategy: find `def ` at or near the start of the trimmed line,
/// then scan the remainder for a bare ` = ` (not `==`, not `=>`) at
/// parenthesis depth 0.
fn is_endless_method(t: &str) -> bool {
    let def_pos = match t.find("def ") {
        // Allow `def ` to appear at position 0–20 (leading spaces already
        // stripped by the caller; small cap is a safety guard).
        Some(p) if p <= 20 => p,
        _ => return false,
    };
    let after = &t[def_pos + 4..];
    let bytes = after.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b' ' if depth == 0
                && i + 2 < bytes.len()
                && bytes[i + 1] == b'='
                // Must not be `==` or `=>`
                && bytes[i + 2] != b'='
                && bytes[i + 2] != b'>' =>
            {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Extract the heredoc terminator word from a line that opens a heredoc.
///
/// Handles `<<~WORD`, `<<-WORD`, and `<<WORD` (bare). Returns `None` if
/// no heredoc opener is found on the line.
///
/// The terminator is the unquoted, stripped identifier after `<<`, `<<-`,
/// or `<<~`.  Quoted heredocs (e.g. `<<"WORD"` or `<<'WORD'`) are also
/// handled by stripping the surrounding quote characters.
fn heredoc_terminator(line: &str) -> Option<String> {
    // Find `<<` in the line.
    let pos = line.find("<<")?;
    let rest = &line[pos + 2..];

    // Strip optional `-` or `~` sigil.
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    let rest = rest.strip_prefix('~').unwrap_or(rest);

    // Strip optional surrounding quotes.
    let rest = if (rest.starts_with('"') && rest.contains('"'))
        || (rest.starts_with('\'') && rest.contains('\''))
        || (rest.starts_with('`') && rest.contains('`'))
    {
        &rest[1..]
    } else {
        rest
    };

    // Collect the identifier (letters, digits, underscores).
    let word: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if word.is_empty() {
        None
    } else {
        Some(word)
    }
}

impl Rule for NestedMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/NestedMethodDefinition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut def_depth = 0usize;
        // When `Some(word)`, we are inside a heredoc whose terminator is `word`.
        let mut heredoc_terminator_word: Option<String> = None;

        for i in 0..n {
            let raw = &lines[i];
            let trimmed = raw.trim_start();
            let t = trimmed.trim();

            // If we are inside a heredoc, look for the terminator line.
            if let Some(ref term) = heredoc_terminator_word {
                if t == term.as_str() {
                    heredoc_terminator_word = None;
                }
                // Either way, skip this line for def/end tracking.
                continue;
            }

            if t.starts_with('#') {
                continue;
            }

            // Check whether this line opens a heredoc; if so, record the
            // terminator so subsequent lines are skipped.
            if let Some(term) = heredoc_terminator(raw) {
                heredoc_terminator_word = Some(term);
                // The opening line itself is still valid Ruby — fall through
                // to check for `def` on the same line (e.g. `def foo <<~MSG`
                // is unusual but technically valid; for safety we do NOT skip
                // the opener line for def/end accounting).
            }

            if t.starts_with("def ") || t == "def" {
                if is_endless_method(t) {
                    // Endless methods (`def foo = expr`) have no `end` token;
                    // do not push depth.
                    continue;
                }

                if def_depth > 0 {
                    // Nested def
                    let indent = raw.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Method defined inside another method.".into(),
                        range: TextRange::new(pos, pos + t.len() as u32),
                        severity: Severity::Warning,
                    });
                }
                def_depth += 1;
            } else if t == "end" && def_depth > 0 {
                def_depth -= 1;
            }
        }

        diags
    }
}
