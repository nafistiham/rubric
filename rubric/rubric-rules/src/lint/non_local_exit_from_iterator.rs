use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NonLocalExitFromIterator;

impl Rule for NonLocalExitFromIterator {
    fn name(&self) -> &'static str {
        "Lint/NonLocalExitFromIterator"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut block_depth = 0usize;
        let mut def_depth = 0usize;

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') {
                continue;
            }

            // Track def openings — a return inside a nested def is not non-local
            if (t.starts_with("def ") || t == "def") && !t.starts_with("defined?") {
                def_depth += 1;
            }

            // Track block openings (do...end blocks that are iterators)
            if def_depth == 0 && (t.contains(" do") || t.contains(" do |") || t.ends_with(" do")) {
                block_depth += 1;
            }

            if t == "end" {
                if def_depth > 0 {
                    def_depth -= 1;
                } else if block_depth > 0 {
                    block_depth -= 1;
                }
            }

            // Inside a block but not inside a nested def, detect `return`
            if block_depth > 0 && def_depth == 0 && (t.starts_with("return ") || t == "return") {
                let indent = lines[i].len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "`return` inside iterator block causes non-local exit.".into(),
                    range: TextRange::new(pos, pos + t.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
