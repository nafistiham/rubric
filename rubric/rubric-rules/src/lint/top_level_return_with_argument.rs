use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TopLevelReturnWithArgument;

/// Returns true if `t` (trimmed line) opens a block that needs a matching `end`,
/// but is NOT a `def` opener (those are tracked separately).
fn opens_non_def_block(t: &str) -> bool {
    // do-blocks (loop do, 3.times do, etc.)
    t.ends_with(" do") || t.contains(" do |") || t.contains(" do\n")
        // Control flow block-form (NOT modifier-form — modifier has keyword after the expression)
        || t.starts_with("if ") || t.starts_with("unless ")
        || t.starts_with("while ") || t.starts_with("until ")
        || t.starts_with("for ")
        || t.starts_with("case ") || t == "case"
        || t == "begin" || t.starts_with("begin ")
        // Class and module openers (not def)
        || t.starts_with("class ") || t.starts_with("class<<") || t.starts_with("class <<")
        || t.starts_with("module ")
}

/// Returns true if `t` is an inline conditional assignment, e.g.:
///   `result = if cond`, `a, b = unless cond`, `x = case val`, `x = begin`
/// These open a block that needs a matching `end` but are not detected by
/// `opens_non_def_block` because the keyword is not at the start of the line.
fn has_inline_conditional(t: &str) -> bool {
    !t.starts_with("def ")
        && !opens_non_def_block(t)
        && (t.contains("= if ")
            || t.ends_with("= if")
            || t.contains("= unless ")
            || t.ends_with("= unless")
            || t.contains("= case ")
            || t.ends_with("= case")
            || t.ends_with("= begin")
            || t.contains("= begin "))
}

impl Rule for TopLevelReturnWithArgument {
    fn name(&self) -> &'static str {
        "Lint/TopLevelReturnWithArgument"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        // def_depth: how many `def` we're nested inside
        let mut def_depth = 0usize;
        // block_depth: how many non-def block openers we're nested inside
        // (do-blocks, if/unless/while/case/begin/class/module)
        let mut block_depth = 0usize;

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') { continue; }

            // Track def openers
            if t.starts_with("def ") { def_depth += 1; }
            // Track other block openers (only when not already a def line)
            else if opens_non_def_block(t) { block_depth += 1; }
            // Inline conditional assignment: `x = if cond`, `a, b = unless cond`, etc.
            // The closing `end` must decrement block_depth, not def_depth.
            else if has_inline_conditional(t) { block_depth += 1; }

            // `end` closes the innermost open block — prefer non-def blocks first
            if t == "end" || t.starts_with("end ") || t.starts_with("end.") {
                if block_depth > 0 {
                    block_depth -= 1;
                } else if def_depth > 0 {
                    def_depth -= 1;
                }
            }

            // At top level (no def, no other block), detect `return <value>`.
            // Trailing `if`/`unless` modifiers are not arguments — skip them.
            let has_return_arg = t.starts_with("return ")
                && t != "return"
                && !t.starts_with("return if ")
                && !t.starts_with("return unless ");
            if def_depth == 0 && block_depth == 0 && has_return_arg {
                let indent = lines[i].len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Top-level `return` with argument; this may cause a `LocalJumpError`.".into(),
                    range: TextRange::new(pos, pos + t.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
