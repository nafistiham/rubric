use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Documentation;

impl Rule for Documentation {
    fn name(&self) -> &'static str {
        "Style/Documentation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("class ") && !trimmed.starts_with("module ") {
                continue;
            }

            // Look backwards (ignoring blank lines) for a comment line
            let mut has_doc = false;
            if i > 0 {
                let mut j = i - 1;
                loop {
                    let prev = lines[j].trim();
                    if prev.is_empty() {
                        if j == 0 { break; }
                        j -= 1;
                        continue;
                    }
                    if prev.starts_with('#') {
                        has_doc = true;
                    }
                    break;
                }
            }

            if !has_doc {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Missing documentation comment for class/module.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
