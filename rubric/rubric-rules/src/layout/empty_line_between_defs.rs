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

        // Scan for `end` on its own line (possibly at any indentation),
        // then look at the very next non-blank line. If it starts with `def`,
        // and there was no blank line in between, flag the second `def`.
        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();
            if trimmed == "end" {
                // Find the next non-blank line
                let mut j = i + 1;
                let mut found_blank = false;
                while j < n {
                    let t = lines[j].trim();
                    if t.is_empty() {
                        found_blank = true;
                        break;
                    }
                    if t.starts_with("def ") || t == "def" {
                        if !found_blank {
                            // Flag the def line
                            let line_start = ctx.line_start_offsets[j];
                            let def_end = line_start + "def".len() as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Missing empty line between method definitions.".into(),
                                range: TextRange::new(line_start, def_end),
                                severity: Severity::Warning,
                            });
                        }
                        break;
                    }
                    // Next non-blank line is not `def`, stop
                    break;
                }
            }
            i += 1;
        }

        diags
    }
}
