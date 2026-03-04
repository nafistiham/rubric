use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RescueEnsureAlignment;

/// Stack frame kinds for tracking block openers.
/// We need to track ALL `end`-consuming constructs to prevent stack corruption,
/// but only check `rescue`/`ensure` alignment against `begin`, `def`, and `do` openers.
#[derive(Clone, Copy, PartialEq)]
enum FrameKind {
    /// `begin`, `def`, or `do` block — `rescue`/`ensure` alignment is checked against these.
    RescueTarget,
    /// `if`, `unless`, `while`, `until`, `for`, `case`, `class`, `module` —
    /// these produce `end` tokens but are NOT rescue targets.
    Other,
}

/// Returns true if `trimmed` starts with the `rescue` keyword (not `rescue_from` etc).
/// The keyword must be followed by whitespace, end of string, `(`, or `#` (comment).
fn starts_with_rescue_keyword(trimmed: &str) -> bool {
    if !trimmed.starts_with("rescue") {
        return false;
    }
    match trimmed.as_bytes().get(6) {
        None => true,              // bare "rescue"
        Some(b' ') | Some(b'\t') | Some(b'(') | Some(b'#') => true,
        _ => false,
    }
}

/// Returns true if `trimmed` starts with the `ensure` keyword (not `ensure_exact_match` etc).
/// The keyword must be followed by whitespace, end of string, or `#` (comment).
fn starts_with_ensure_keyword(trimmed: &str) -> bool {
    if !trimmed.starts_with("ensure") {
        return false;
    }
    match trimmed.as_bytes().get(6) {
        None => true,              // bare "ensure"
        Some(b' ') | Some(b'\t') | Some(b'#') => true,
        _ => false,
    }
}

/// Returns true if the line is an inline begin assignment of the form:
///   `lhs = begin`, `lhs ||= begin`, `lhs &&= begin`, etc.
/// Also catches `lhs = begin` inside any expression (line ends with ` begin`
/// and is not a standalone `begin`).
fn is_inline_begin_opener(trimmed: &str) -> bool {
    // Must not be a standalone `begin` statement (which already matches starts_with("begin")).
    if trimmed == "begin" || trimmed.starts_with("begin ") || trimmed.starts_with("begin#") {
        return false;
    }
    // The line ends with ` begin` (inline begin as an expression value).
    trimmed.ends_with(" begin")
}

/// Returns true if the line contains an inline `if`/`unless`/`case` opener
/// that is NOT at the start of the line (i.e., it's an expression value, not
/// a standalone conditional block). These produce `end` tokens that must be
/// tracked to avoid stack corruption.
///
/// Patterns detected:
/// - `lhs = if cond`, `lhs = unless cond`, `lhs = case val`
/// - `lhs ||= if cond`, `lhs &&= if cond`
fn is_inline_conditional_opener(trimmed: &str) -> bool {
    // Already handled by is_other_opener if it starts with the keyword — skip those.
    for kw in &["if ", "unless ", "case "] {
        if trimmed.starts_with(kw) {
            return false;
        }
    }
    // Look for `= if`, `= unless`, `= case` assignment patterns.
    // Also handle compound: `||= if`, `&&= if`.
    // We use a simple scan: look for ` = if `, ` = unless `, ` = case ` etc.
    for kw in &["if", "unless", "case"] {
        // Look for ` if `, ` if\n` (at EOL, body on next line) preceded by `=`
        let pattern_mid = format!("= {} ", kw);
        let pattern_eol = format!("= {}", kw);
        if trimmed.contains(&pattern_mid)
            || (trimmed.ends_with(&pattern_eol) && {
                // Ensure it's really ` = kw` not part of an identifier.
                if let Some(pos) = trimmed.rfind(&pattern_eol) {
                    // Check the char before `=`: must not be `=` (avoid `==`)
                    let before = trimmed[..pos].trim_end();
                    !before.ends_with('=') && !before.ends_with('!')
                } else {
                    false
                }
            })
        {
            return true;
        }
    }
    false
}

impl Rule for RescueEnsureAlignment {
    fn name(&self) -> &'static str {
        "Layout/RescueEnsureAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack of (indent, kind) for all block openers.
        // We track everything that produces an `end` token so that `end` tokens
        // for `if/unless/case/etc` don't accidentally pop `begin/def/do` frames.
        let mut stack: Vec<(usize, FrameKind)> = Vec::new();

        let mut i = 0;
        while i < n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Detect `do` block openers: `foo do`, `foo do |x|`, etc.
            let opens_do_block = (trimmed.ends_with(" do")
                || trimmed.contains(" do |")
                || trimmed.contains(" do|"))
                && !trimmed.starts_with("def ")
                && trimmed != "def";

            // Detect begin/def/do openers — these are rescue targets.
            // Also includes inline begin assignments: `var = begin`, `var ||= begin` etc.
            let is_rescue_target_opener = trimmed.starts_with("begin")
                || trimmed.starts_with("def ")
                || trimmed == "def"
                || opens_do_block
                || is_inline_begin_opener(trimmed);

            // Detect other openers that consume `end` but are NOT rescue targets.
            // We must track these to avoid stack corruption when their `end` appears.
            // Includes standalone block openers (if/unless/etc at line start) AND
            // inline conditional openers (lhs = if cond, lhs = case val, etc.).
            let is_other_opener = !is_rescue_target_opener && (
                trimmed.starts_with("if ")
                || trimmed == "if"
                || trimmed.starts_with("unless ")
                || trimmed == "unless"
                || trimmed.starts_with("while ")
                || trimmed == "while"
                || trimmed.starts_with("until ")
                || trimmed == "until"
                || trimmed.starts_with("for ")
                || trimmed.starts_with("case ")
                || trimmed == "case"
                || trimmed.starts_with("class ")
                || trimmed == "class"
                || trimmed.starts_with("module ")
                || trimmed == "module"
                || is_inline_conditional_opener(trimmed)
            );

            if is_rescue_target_opener {
                stack.push((indent, FrameKind::RescueTarget));
            } else if is_other_opener {
                stack.push((indent, FrameKind::Other));
            }

            // Detect `end` token (bare `end` or `end` followed by non-identifier char).
            let is_end = {
                let t = trimmed;
                t == "end"
                    || (t.starts_with("end") && {
                        let next = t.as_bytes().get(3).copied();
                        match next {
                            Some(c) => !c.is_ascii_alphanumeric() && c != b'_',
                            None => false,
                        }
                    })
            };

            if is_end {
                stack.pop();
            }

            // Check `rescue`/`ensure` alignment: compare against the nearest
            // RescueTarget frame (skip Other frames).
            // Use keyword-aware checks to avoid false positives from method calls
            // like `rescue_from` or `ensure_exact_match`.
            let is_rescue = starts_with_rescue_keyword(trimmed);
            let is_ensure = starts_with_ensure_keyword(trimmed);

            if is_rescue || is_ensure {
                // Find the nearest RescueTarget frame on the stack.
                let expected_indent = stack
                    .iter()
                    .rev()
                    .find(|(_, kind)| *kind == FrameKind::RescueTarget)
                    .map(|(ind, _)| *ind);

                if let Some(expected_indent) = expected_indent {
                    if indent != expected_indent {
                        let line_start = ctx.line_start_offsets[i];
                        let keyword_end = if is_rescue {
                            line_start + indent as u32 + "rescue".len() as u32
                        } else {
                            line_start + indent as u32 + "ensure".len() as u32
                        };
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "`rescue`/`ensure` indentation ({}) does not match its opener ({}).",
                                indent, expected_indent
                            ),
                            range: TextRange::new(line_start + indent as u32, keyword_end),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            i += 1;
        }

        diags
    }
}
