use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AccessorMethodName;

impl Rule for AccessorMethodName {
    fn name(&self) -> &'static str {
        "Naming/AccessorMethodName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Must start with `def `
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest
            } else {
                continue;
            };

            // Skip self. prefix
            let method_part = after_def.strip_prefix("self.").unwrap_or(after_def);

            // Extract method name up to (, space, tab, or end
            let name_end = method_part
                .find(|c: char| c == '(' || c == ' ' || c == '\t' || c == '\n')
                .unwrap_or(method_part.len());
            let method_name = &method_part[..name_end];

            if method_name.is_empty() {
                continue;
            }

            let (flagged, msg) = if method_name.starts_with("get_") && method_name.len() > 4 {
                (true, format!("Do not prefix reader method names with `get_`."))
            } else if method_name.starts_with("set_") && method_name.len() > 4 {
                (true, format!("Do not prefix writer method names with `set_`."))
            } else {
                (false, String::new())
            };

            if flagged {
                let line_start = ctx.line_start_offsets[i] as usize;
                let def_col = line.len() - trimmed.len();
                let start = (line_start + def_col) as u32;
                let end = start + 3; // `def`
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: msg,
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
