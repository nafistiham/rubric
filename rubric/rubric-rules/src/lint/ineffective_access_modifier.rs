use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IneffectiveAccessModifier;

impl Rule for IneffectiveAccessModifier {
    fn name(&self) -> &'static str {
        "Lint/IneffectiveAccessModifier"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            let trimmed = lines[i].trim();
            // Check for access modifier on its own line
            let is_access_modifier = trimmed == "private" || trimmed == "protected" || trimmed == "public";
            if !is_access_modifier {
                continue;
            }

            // Look ahead for `def self.`
            let mut j = i + 1;
            while j < n {
                let next = lines[j].trim();
                if next.is_empty() || next.starts_with('#') {
                    j += 1;
                    continue;
                }
                if next.starts_with("def self.") {
                    let indent = lines[j].len() - lines[j].trim_start().len();
                    let line_start = ctx.line_start_offsets[j] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "`{}` has no effect on `def self.` class methods.",
                            trimmed
                        ),
                        range: TextRange::new(pos, pos + next.len() as u32),
                        severity: Severity::Warning,
                    });
                }
                break;
            }
        }

        diags
    }
}
