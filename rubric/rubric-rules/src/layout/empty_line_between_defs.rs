use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLineBetweenDefs;

impl Rule for EmptyLineBetweenDefs {
    fn name(&self) -> &'static str {
        "Layout/EmptyLineBetweenDefs"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Scan for `end` on its own line, then check the very next line.
        // If it's a `def` with no blank line between, flag the second `def`.
        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();
            if trimmed == "end" && i + 1 < n {
                let next = lines[i + 1].trim();
                // If the next line is directly a `def`, flag it (no blank line in between)
                if next.starts_with("def ") || next == "def" {
                    let j = i + 1;
                    let line_start = ctx.line_start_offsets[j];
                    let def_end = line_start + "def".len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Missing empty line between method definitions.".into(),
                        range: TextRange::new(line_start, def_end),
                        severity: Severity::Warning,
                    });
                }
            }
            i += 1;
        }

        diags
    }
}
