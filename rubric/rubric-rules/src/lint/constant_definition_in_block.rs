use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ConstantDefinitionInBlock;

impl Rule for ConstantDefinitionInBlock {
    fn name(&self) -> &'static str {
        "Lint/ConstantDefinitionInBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track block depth (do...end)
        let mut block_depth = 0usize;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Track block openings
            if trimmed.contains(" do") || trimmed == "do" || trimmed.contains(" do |") {
                block_depth += 1;
            }

            if trimmed == "end" && block_depth > 0 {
                block_depth -= 1;
            }

            // Inside a block, detect constant assignment
            if block_depth > 0 {
                // Constant: starts with uppercase letter followed by more uppercase/digit/underscore
                // Pattern: `CONST_NAME = value`
                let t = trimmed;
                if !t.is_empty() {
                    let first = t.chars().next().unwrap_or(' ');
                    if first.is_ascii_uppercase() {
                        // Check if it looks like a constant assignment
                        if let Some(eq_pos) = t.find(" = ").or_else(|| t.find("=")) {
                            let lhs = &t[..eq_pos];
                            // LHS should be all uppercase/digits/underscores
                            if lhs.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                                let indent = line.len() - trimmed.len();
                                let line_start = ctx.line_start_offsets[i] as usize;
                                let pos = (line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Do not define constant `{}` inside a block.",
                                        lhs
                                    ),
                                    range: TextRange::new(pos, pos + lhs.len() as u32),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }
        }

        diags
    }
}
