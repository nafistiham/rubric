use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ProcNew;

impl Rule for ProcNew {
    fn name(&self) -> &'static str {
        "Style/Proc"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Find `Proc.new` pattern
            let mut search: &str = line;
            let mut offset = 0usize;
            while let Some(pos) = search.find("Proc.new") {
                let line_start = ctx.line_start_offsets[i] as usize;
                let abs_start = (line_start + offset + pos) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `proc` instead of `Proc.new`.".into(),
                    range: TextRange::new(abs_start, abs_start + "Proc.new".len() as u32),
                    severity: Severity::Warning,
                });
                offset += pos + "Proc.new".len();
                search = &line[offset..];
            }
        }

        diags
    }
}
