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

            // Extract the parameter list between ( and ).
            // Use depth tracking to find the MATCHING close paren, not the last `)`.
            // This avoids false matches in endless methods like `def foo(a) = bar(b)`.
            let Some(open_paren) = line.find('(') else { continue; };
            let bytes = line.as_bytes();
            let mut depth = 0i32;
            let mut close_paren = None;
            for (idx, &b) in bytes.iter().enumerate().skip(open_paren) {
                match b {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            close_paren = Some(idx);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let Some(close_paren) = close_paren else { continue; };
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
                // Keyword argument: `name: value` or `name:` — word chars followed by `:`
                // Keyword arguments are not positional, so they don't violate this rule.
                let is_keyword_arg = p.find(':').map(|colon_pos| {
                    colon_pos > 0 && p[..colon_pos].chars().all(|c| c.is_alphanumeric() || c == '_')
                }).unwrap_or(false);
                if is_keyword_arg {
                    continue;
                }
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
