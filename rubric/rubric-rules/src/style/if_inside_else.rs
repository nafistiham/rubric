use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IfInsideElse;

impl Rule for IfInsideElse {
    fn name(&self) -> &'static str {
        "Style/IfInsideElse"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for i in 0..ctx.lines.len().saturating_sub(1) {
            let current = ctx.lines[i].trim();
            let next = ctx.lines[i + 1].trim_start();

            if current == "else" && next.starts_with("if ") {
                let line_start = ctx.line_start_offsets[i] as usize;
                let start = line_start as u32;
                let end = start + ctx.lines[i].len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Convert if inside else to elsif.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
