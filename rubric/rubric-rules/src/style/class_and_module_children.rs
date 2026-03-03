use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClassAndModuleChildren;

impl Rule for ClassAndModuleChildren {
    fn name(&self) -> &'static str {
        "Style/ClassAndModuleChildren"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let is_class = trimmed.starts_with("class ");
            let is_module = trimmed.starts_with("module ");

            if !is_class && !is_module {
                continue;
            }

            let rest = if is_class {
                &trimmed["class ".len()..]
            } else {
                &trimmed["module ".len()..]
            };

            // Detect compact notation: ClassName::SubName
            // Only check the class/module NAME itself, not the parent class after `<`.
            // e.g. `class Foo::Bar` → compact (flag)
            //      `class Foo < A::B` → parent class ref only (skip)
            let name_part = if is_class {
                rest.split_once(" < ").map(|(n, _)| n).unwrap_or(rest)
            } else {
                rest
            };
            if name_part.contains("::") {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use nested class/module definitions instead of compact notation.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
