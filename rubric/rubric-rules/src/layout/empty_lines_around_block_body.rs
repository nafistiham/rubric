use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLinesAroundBlockBody;

impl Rule for EmptyLinesAroundBlockBody {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundBlockBody"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            let trimmed = lines[i].trim();

            // Detect `do` at end of line or `do |...|` — block opened with `do`
            let opens_do_block = trimmed.ends_with(" do")
                || trimmed == "do"
                || (trimmed.contains(" do |") || trimmed.contains(" do|"));

            if opens_do_block {
                // Check if the next line is blank
                if i + 1 < n && lines[i + 1].trim().is_empty() {
                    let blank_line = i + 1;
                    let line_start = ctx.line_start_offsets[blank_line] as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Extra empty line detected after `do` block opening.".into(),
                        range: TextRange::new(line_start, line_start + lines[blank_line].len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            // Detect `end` line that closes a block — check line before it
            if trimmed == "end" && i > 0 && lines[i - 1].trim().is_empty() {
                let blank_line = i - 1;
                let line_start = ctx.line_start_offsets[blank_line] as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Extra empty line detected before block `end`.".into(),
                    range: TextRange::new(line_start, line_start + lines[blank_line].len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
