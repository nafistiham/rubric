use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct OptionalArguments;

impl Rule for OptionalArguments {
    fn name(&self) -> &'static str {
        "Style/OptionalArguments"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("def ") {
                continue;
            }

            // Extract the parameter list between ( and )
            let Some(open_paren) = line.find('(') else { continue; };
            let Some(close_paren) = line.rfind(')') else { continue; };
            if close_paren <= open_paren {
                continue;
            }

            let params_str = &line[open_paren + 1..close_paren];
            // Split by comma (simple approach — doesn't handle nested parens)
            let params: Vec<&str> = params_str.split(',').collect();

            // Track if we've seen an optional param (one with `=`)
            let mut seen_optional = false;
            let mut violation_pos: Option<usize> = None;

            for param in &params {
                let p = param.trim();
                if p.contains('=') {
                    seen_optional = true;
                } else if seen_optional && !p.is_empty() && !p.starts_with('*') && !p.starts_with('&') {
                    // Required argument after optional
                    if violation_pos.is_none() {
                        // Find position of this param in the line
                        if let Some(pos) = line.find(p) {
                            violation_pos = Some(pos);
                        }
                    }
                }
            }

            if let Some(pos) = violation_pos {
                let line_start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Required argument after optional argument.".into(),
                    range: TextRange::new(line_start + pos as u32, line_start + pos as u32 + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
