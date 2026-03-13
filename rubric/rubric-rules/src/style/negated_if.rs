use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NegatedIf;

/// Returns `true` when the condition text (everything after `if !`) is a
/// compound boolean expression joined by `&&` or `||`.
///
/// RuboCop's `NegatedIf` only flags a *single* negated condition because a
/// compound expression cannot be mechanically rewritten as `unless`.  For
/// example `if !a && b` cannot be naively transformed to `unless a && b`
/// (De Morgan would require `unless a || !b`), so RuboCop leaves such lines
/// alone.
///
/// The heuristic used here checks:
/// 1. Inline compound: ` && ` or ` || ` surrounded by spaces (the common case).
/// 2. Trailing operator: the condition text ends with ` &&` or ` ||` (indicating
///    the expression continues on the next line — still compound).
fn is_compound_condition(condition: &str) -> bool {
    let trimmed = condition.trim_end();
    // Inline compound operators with spaces on both sides
    if trimmed.contains(" && ") || trimmed.contains(" || ") {
        return true;
    }
    // Trailing operator — expression spans multiple lines
    if trimmed.ends_with(" &&") || trimmed.ends_with(" ||") {
        return true;
    }
    false
}

/// Return true when the `if` block starting at line `start_line` (with indent
/// `block_indent`) contains an `elsif` branch.  We look ahead until we find
/// the matching `end` (or EOF), stopping at `elsif` at the same indent level.
/// This prevents flagging `if !cond ... elsif ...` since `unless` cannot have
/// an `elsif` branch in Ruby.
fn block_has_elsif(lines: &[&str], start_line: usize, block_indent: usize) -> bool {
    let mut depth: i32 = 1; // we are already inside the `if` block
    let mut j = start_line + 1;
    while j < lines.len() && depth > 0 {
        let t = lines[j].trim_start();
        let t_indent = lines[j].len() - t.len();
        // Only look at `elsif` at the same indentation level.
        if t_indent == block_indent {
            if t.starts_with("elsif ") || t == "elsif" {
                return true;
            }
            if t == "end" || t.starts_with("end ") || t.starts_with("end.") {
                depth -= 1;
                j += 1;
                continue;
            }
        }
        // Track depth changes for nested blocks (simplified: only `if`/`unless`/`do` etc.).
        if t.starts_with("if ") || t.starts_with("unless ") || t.starts_with("case ")
            || t.starts_with("begin") || t.starts_with("def ") || t.starts_with("do ")
            || t.ends_with(" do") || t.ends_with(" do |")
        {
            depth += 1;
        }
        if t == "end" || t.starts_with("end ") || t.starts_with("end.") {
            depth -= 1;
        }
        j += 1;
    }
    false
}

impl Rule for NegatedIf {
    fn name(&self) -> &'static str {
        "Style/NegatedIf"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Skip full-line comments.
            if trimmed.starts_with('#') {
                continue;
            }

            // ── Block-form: `if !condition` where `if` is the first token ──
            if trimmed.starts_with("if ") {
                let after_if = trimmed["if ".len()..].trim_start();
                if after_if.starts_with('!') {
                    // Skip compound conditions — cannot be mechanically rewritten
                    // with `unless` (RuboCop leaves these alone).
                    if is_compound_condition(after_if) {
                        continue;
                    }
                    // Skip when the block has an `elsif` branch — Ruby has no
                    // `unless ... elsif`, so the conversion is impossible and
                    // RuboCop leaves such blocks alone.
                    if block_has_elsif(&ctx.lines, i, indent) {
                        continue;
                    }
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `unless` instead of `if !`.".into(),
                        range: TextRange::new(pos, pos + 2),
                        severity: Severity::Warning,
                    });
                    continue; // block-form found; no need to check modifier-form too
                }
            }

            // ── Modifier-form: `expression if !condition` ──────────────────
            // Search for ` if !` anywhere in the trimmed line.  ` if !` (with
            // a leading space) never matches the block-form case (which starts
            // with `if`, no leading space in the trimmed text).
            if let Some(rel_pos) = trimmed.find(" if !") {
                // Condition starts right after `if !` (4 chars: space+if+space)
                let condition_start = rel_pos + " if !".len();
                let condition = &trimmed[condition_start..];
                // Skip compound conditions — same reasoning as block-form.
                if is_compound_condition(condition) {
                    continue;
                }
                let pos = (line_start + indent + rel_pos + 1) as u32; // +1: skip leading space
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `unless` instead of `if !`.".into(),
                    range: TextRange::new(pos, pos + 2),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
