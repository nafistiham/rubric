use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct GuardClause;

impl Rule for GuardClause {
    fn name(&self) -> &'static str {
        "Style/GuardClause"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect three consecutive lines matching:
        // 1. Line matches `^\s+if\s+` (indented `if`)
        // 2. Next line is the body (one line, non-empty)
        // 3. Line after that is `^\s+end\s*$`
        for i in 0..n.saturating_sub(2) {
            let line_if = lines[i].trim_start();
            let line_body = lines[i+1].trim_start();
            let line_end = lines[i+2].trim_start();

            if !line_if.starts_with("if ") {
                continue;
            }

            if line_body.is_empty() {
                continue;
            }

            if line_end.trim_end() != "end" {
                continue;
            }

            // Confirm the `if` is indented (not at the top level — this avoids top-level ifs)
            let indent = lines[i].len() - line_if.len();
            if indent == 0 {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: "Consider using a guard clause (`return unless condition`) instead of wrapping in `if`.".into(),
                range: TextRange::new(line_start + indent as u32, line_start + indent as u32 + 2),
                severity: Severity::Warning,
            });
        }

        diags
    }
}
