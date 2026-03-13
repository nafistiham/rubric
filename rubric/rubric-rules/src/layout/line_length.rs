use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub const DEFAULT_MAX: usize = 120;

pub struct LineLength {
    pub max: usize,
}

impl Default for LineLength {
    fn default() -> Self {
        Self { max: DEFAULT_MAX }
    }
}

impl Rule for LineLength {
    fn name(&self) -> &'static str {
        "Layout/LineLength"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let max = self.max;
        for (i, line) in ctx.lines.iter().enumerate() {
            if line.len() > max {
                let start = ctx.line_start_offsets[i];
                let end = start + line.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Line is {} characters (max is {max}).", line.len()),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }
        diags
    }
}
