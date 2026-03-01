use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NestedMethodDefinition;

impl Rule for NestedMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/NestedMethodDefinition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut def_depth = 0usize;

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') {
                continue;
            }

            if t.starts_with("def ") || t == "def" {
                if def_depth > 0 {
                    // Nested def
                    let indent = lines[i].len() - trimmed.len();
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
