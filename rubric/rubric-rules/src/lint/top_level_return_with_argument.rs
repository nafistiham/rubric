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

            // `end` closes the innermost open block — prefer non-def blocks first
            if t == "end" || t.starts_with("end ") || t.starts_with("end.") {
                if block_depth > 0 {
                    block_depth -= 1;
                } else if def_depth > 0 {
                    def_depth -= 1;
                }
            }

            // At top level (no def, no other block), detect `return <value>`
            if def_depth == 0 && block_depth == 0 && t.starts_with("return ") && t != "return" {
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
