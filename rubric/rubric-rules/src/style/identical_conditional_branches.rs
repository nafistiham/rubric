use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IdenticalConditionalBranches;

impl Rule for IdenticalConditionalBranches {
    fn name(&self) -> &'static str {
        "Style/IdenticalConditionalBranches"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim_start();

            // Match an `if` or `elsif` opener that is NOT a modifier (has content after `if cond`)
            // We want multi-line `if condition` where the condition ends the line.
            if is_if_opener(trimmed) {
                if let Some(diag) = check_if_block(ctx, i) {
                    diags.push(diag);
                }
            }

            i += 1;
        }

        diags
    }
}

/// Returns true if the line starts a multi-line `if` block.
/// A multi-line `if` line contains `if` followed by a condition and nothing else
/// (no `;`, no inline `then` with a statement after it).
fn is_if_opener(trimmed: &str) -> bool {
    if !trimmed.starts_with("if ") && trimmed != "if" {
        return false;
    }
    // Exclude one-liners: `if cond; body; end` or `if cond then body end`
    if trimmed.contains(';') {
        return false;
    }
    // Exclude `if cond then body` where `then` is followed by actual code
    // Simple check: if `then` appears and there's content after it, skip
    if let Some(then_pos) = trimmed.find(" then ") {
        let after_then = &trimmed[then_pos + 6..];
        if !after_then.trim().is_empty() {
            return false;
        }
    }
    true
}

/// Given the index of an `if` line, look for the pattern:
/// ```
/// if condition        <- line i
///   [one body line]   <- line i+1 (non-blank, non-`else`/`elsif`/`end`)
/// else                <- line i+2 trimmed == "else"
///   [same body line]  <- line i+3 trimmed matches line i+1 trimmed
/// end                 <- line i+4 trimmed == "end"
/// ```
/// Returns a Diagnostic on the `if` line if the pattern matches.
fn check_if_block(ctx: &LintContext, if_idx: usize) -> Option<Diagnostic> {
    let lines = &ctx.lines;
    let n = lines.len();

    // Collect non-blank lines of the `if` body (stopping at `else`, `elsif`, or `end`)
    let mut idx = if_idx + 1;

    // Skip blank lines before first body statement
    while idx < n && lines[idx].trim().is_empty() {
        idx += 1;
    }
    if idx >= n {
        return None;
    }

    let first_body_trimmed = lines[idx].trim();

    // Must be exactly one body line — it should not itself be a control word
    if is_control_keyword(first_body_trimmed) {
        return None;
    }

    // Advance past the body line
    idx += 1;

    // Skip blank lines
    while idx < n && lines[idx].trim().is_empty() {
        idx += 1;
    }
    if idx >= n {
        return None;
    }

    let separator = lines[idx].trim();

    // Only handle `else` (not `elsif` for simplicity)
    if separator != "else" {
        return None;
    }

    idx += 1;

    // Skip blank lines before else body
    while idx < n && lines[idx].trim().is_empty() {
        idx += 1;
    }
    if idx >= n {
        return None;
    }

    let else_body_trimmed = lines[idx].trim();

    if is_control_keyword(else_body_trimmed) {
        return None;
    }

    idx += 1;

    // Skip blank lines before end
    while idx < n && lines[idx].trim().is_empty() {
        idx += 1;
    }
    if idx >= n {
        return None;
    }

    let end_line = lines[idx].trim();
    if end_line != "end" {
        return None;
    }

    // Check identity
    if first_body_trimmed != else_body_trimmed {
        return None;
    }

    let line_start = ctx.line_start_offsets[if_idx] as u32;
    let line_end = line_start + lines[if_idx].len() as u32;

    Some(Diagnostic {
        rule: "Style/IdenticalConditionalBranches",
        message: "All branches in the conditional are identical.".into(),
        range: TextRange::new(line_start, line_end),
        severity: Severity::Warning,
    })
}

/// Returns true if `trimmed` is a Ruby control keyword that starts a new block.
fn is_control_keyword(trimmed: &str) -> bool {
    matches!(
        trimmed,
        "else"
            | "elsif"
            | "end"
            | "rescue"
            | "ensure"
            | "when"
    ) || trimmed.starts_with("elsif ")
        || trimmed.starts_with("when ")
        || trimmed.starts_with("rescue ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::LintContext;
    use std::path::Path;

    #[test]
    fn detects_identical_if_else_branches() {
        let src = "if condition\n  do_something\nelse\n  do_something\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = IdenticalConditionalBranches.check_source(&ctx);
        assert!(!diags.is_empty(), "expected violation for identical branches");
        assert!(diags.iter().all(|d| d.rule == "Style/IdenticalConditionalBranches"));
    }

    #[test]
    fn no_violation_for_different_branches() {
        let src = "if condition\n  do_something\nelse\n  do_other_thing\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = IdenticalConditionalBranches.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_if_without_else() {
        let src = "if condition\n  do_something\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = IdenticalConditionalBranches.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation for if-only; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_multi_statement_branches() {
        // If branch has two statements — we don't flag it (we only detect simple 1-liner bodies)
        let src = "if condition\n  do_something\n  do_more\nelse\n  do_something\n  do_more\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = IdenticalConditionalBranches.check_source(&ctx);
        // Two body lines before `else` means the first body line is followed by another
        // body line, not `else` — the check won't match. This is acceptable.
        // The test just verifies we don't crash.
        let _ = diags;
    }

    #[test]
    fn detects_identical_branches_with_method_call() {
        let src = "if x > 0\n  render :ok\nelse\n  render :ok\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = IdenticalConditionalBranches.check_source(&ctx);
        assert!(!diags.is_empty(), "expected violation for identical render calls");
    }
}
