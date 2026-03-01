use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct WhileUntilModifier;

impl Rule for WhileUntilModifier {
    fn name(&self) -> &'static str {
        "Style/WhileUntilModifier"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect 3-line pattern: `while/until <cond>` / `  <single_stmt>` / `end`
        let mut i = 0;
        while i + 2 < n {
            let line0 = lines[i].trim();
            let line1 = lines[i + 1].trim();
            let line2 = lines[i + 2].trim();

            let is_while = line0.starts_with("while ") || line0 == "while";
            let is_until = line0.starts_with("until ") || line0 == "until";

            if (is_while || is_until) && line2 == "end" {
                let adds_depth = line1.starts_with("if ") || line1.starts_with("unless ")
                    || line1.starts_with("while ") || line1.starts_with("until ")
                    || line1.starts_with("for ") || line1.starts_with("def ")
                    || line1.starts_with("class ") || line1.starts_with("module ")
                    || line1.starts_with("begin") || line1.ends_with(" do")
                    || line1.is_empty();

                if !adds_depth {
                    let line_start = ctx.line_start_offsets[i];
                    let keyword_len = 5u32; // "while" and "until" are both 5 chars
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use modifier form instead of multi-line `while`/`until`.".into(),
                        range: TextRange::new(line_start, line_start + keyword_len),
                        severity: Severity::Warning,
                    });
                }
            }
            i += 1;
        }

        diags
    }
}
