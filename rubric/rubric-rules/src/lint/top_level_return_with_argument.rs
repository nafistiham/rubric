use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TopLevelReturnWithArgument;

impl Rule for TopLevelReturnWithArgument {
    fn name(&self) -> &'static str {
        "Lint/TopLevelReturnWithArgument"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut def_depth = 0usize;

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') { continue; }

            if t.starts_with("def ") { def_depth += 1; }
            if t == "end" && def_depth > 0 { def_depth -= 1; }

            // At top level (depth 0), detect `return <value>`
            if def_depth == 0 && t.starts_with("return ") && t != "return" {
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
