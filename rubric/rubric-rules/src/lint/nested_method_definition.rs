use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NestedMethodDefinition;

/// Tracks what kind of block is currently open.
#[derive(Debug, PartialEq)]
enum FrameKind {
    /// An opened `def ... end` block.
    Def,
    /// Any other block: do-block, class, module, if, unless, while, until,
    /// for, case, begin, etc.
    Other,
}

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

/// Returns `true` when the trimmed line is a single-line method definition that
/// opens and closes on the same line, e.g. `def setup; body; end`.
///
/// These one-liners never push to the stack because their `end` token is on
/// the same line as the `def` — the line-by-line scanner would otherwise miss
/// the closing `end` and leave a frame permanently on the stack.
fn is_one_liner_def(t: &str) -> bool {
    // Must contain "; end" somewhere after the def signature.
    // Also accept the rare "; end " (end followed by a comment marker).
    t.contains("; end") || t.contains(";end")
}

/// Returns `true` when the trimmed line opens a non-def block that requires a
/// matching `end`: do-blocks, class, module, if, unless, while, until, for,
/// case, begin.
///
/// Single-line class/module (`class Foo; end`) are excluded because their
/// closing `end` appears on the same line.
fn opens_other_block(t: &str) -> bool {
    // do-block: ends with ` do`, contains ` do |`, ` do|`, or is exactly `do`
    if t == "do"
        || t.ends_with(" do")
        || t.contains(" do |")
        || t.contains(" do|")
    {
        return true;
    }

    // Keyword-based openers
    let keyword_match = t.starts_with("if ")
        || t.starts_with("unless ")
        || t.starts_with("while ")
        || t.starts_with("until ")
        || t.starts_with("for ")
        || t.starts_with("case ")
        || t == "begin"
        || t.starts_with("begin ");

    if keyword_match {
        return true;
    }

    // class / module — but NOT one-liners like `class Foo; end`
    if t.starts_with("class ") || t.starts_with("module ") {
        // One-liner: contains `; end` (with or without trailing content)
        if t.contains("; end") || t.contains(";end") {
            return false;
        }
        return true;
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
        // Frame stack: each opened block (def or other) pushes a frame.
        // A `def` is only flagged as nested when the TOP frame is `Def`
        // (i.e. directly inside another def, not inside an intervening block).
        let mut stack: Vec<FrameKind> = Vec::new();
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
                    // do not push a frame.
                    continue;
                }

                // Single-line `def method; body; end` — the `end` is on the same
                // line as the `def`. The line-by-line scan would never see the
                // closing `end`, leaving a frame permanently on the stack. Skip
                // frame changes entirely for these one-liners.
                if is_one_liner_def(t) {
                    continue;
                }

                // Flag only when directly inside another def (top frame is Def).
                if stack.last() == Some(&FrameKind::Def) {
                    // Nested def — flag it (unless it's a singleton method def `def obj.method`)
                    // Skip singleton method definitions like `def self.foo` or `def obj.foo`
                    let after_def = t.strip_prefix("def ").unwrap_or("");
                    let is_singleton = after_def.contains('.') && !after_def.starts_with('(');

                    if !is_singleton {
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
                }
                stack.push(FrameKind::Def);
            } else if t == "end"
                || (t.starts_with("end") && matches!(
                    t.as_bytes().get(3).copied(),
                    Some(c) if !c.is_ascii_alphanumeric() && c != b'_' && c != b':'
                ))
            {
                // Pop the top frame (if any). Do not push anything.
                // The word-boundary check (not alphanumeric, not _, not :) prevents
                // `endless` / `end:` from being treated as `end` tokens.
                stack.pop();
            } else if opens_other_block(t) {
                stack.push(FrameKind::Other);
            }
        }

        diags
    }
}
