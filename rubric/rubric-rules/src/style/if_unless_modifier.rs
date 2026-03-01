use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct IfUnlessModifier;

impl Rule for IfUnlessModifier {
    fn name(&self) -> &'static str {
        "Style/IfUnlessModifier"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Detect 3-line pattern: `if/unless <cond>` / `  <single_stmt>` / `end`
        let mut i = 0;
        while i + 2 < n {
            let line0 = lines[i].trim();
            let line1 = lines[i + 1].trim();
            let line2 = lines[i + 2].trim();

            let is_if = line0.starts_with("if ") || line0 == "if";
            let is_unless = line0.starts_with("unless ") || line0 == "unless";

            if (is_if || is_unless) && line2 == "end" {
                // line1 must be a single statement (not contain keywords that add depth)
                let adds_depth = line1.starts_with("if ") || line1.starts_with("unless ")
                    || line1.starts_with("while ") || line1.starts_with("until ")
                    || line1.starts_with("for ") || line1.starts_with("def ")
                    || line1.starts_with("class ") || line1.starts_with("module ")
                    || line1.starts_with("begin") || line1.ends_with(" do")
                    || line1 == "else" || line1 == "elsif" || line1.starts_with("elsif ")
                    || line1.is_empty();

                if !adds_depth {
                    let line_start = ctx.line_start_offsets[i];
                    let keyword_len = if is_if { 2 } else { 6 }; // "if" or "unless"
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use modifier form instead of multi-line `if`/`unless`.".into(),
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
