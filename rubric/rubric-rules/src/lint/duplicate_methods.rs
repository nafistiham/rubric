use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashMap;

pub struct DuplicateMethods;

impl Rule for DuplicateMethods {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Map: method_name -> first line index, scoped per class/module
        let mut seen: HashMap<String, usize> = HashMap::new();

        for i in 0..n {
            let trimmed = lines[i].trim_start();

            // Reset seen map when entering a new class or module scope
            let t = trimmed.trim();
            if t.starts_with("class ") || t.starts_with("module ") {
                seen.clear();
                continue;
            }

            if !trimmed.starts_with("def ") {
                continue;
            }

            // Extract method name
            let after_def = &trimmed["def ".len()..];
            let name_end = after_def
                .find(|c: char| c == '(' || c == ' ' || c == '\n')
                .unwrap_or(after_def.len());
            let method_name = &after_def[..name_end];

            if method_name.is_empty() {
                continue;
            }

            if let Some(&first_line) = seen.get(method_name) {
                let indent = lines[i].len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Duplicate method `{}` (first defined at line {}).",
                        method_name,
                        first_line + 1
                    ),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            } else {
                seen.insert(method_name.to_string(), i);
            }
        }

        diags
    }
}
