use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StructInheritance;

impl Rule for StructInheritance {
    fn name(&self) -> &'static str {
        "Style/StructInheritance"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `class Foo < Struct.new`
            if trimmed.starts_with("class ") && trimmed.contains("< Struct.new") {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `Foo = Struct.new(...)` instead of `class Foo < Struct.new(...)`.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
