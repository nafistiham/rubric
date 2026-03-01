use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct DuplicateRequire;

impl Rule for DuplicateRequire {
    fn name(&self) -> &'static str {
        "Lint/DuplicateRequire"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Match `require '...'` or `require_relative '...'`
            let require_val = extract_require(trimmed);

            if let Some(val) = require_val {
                if !seen.insert(val.to_string()) {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!("Duplicate `require` for `{}`.", val),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}

fn extract_require(line: &str) -> Option<&str> {
    let prefixes = ["require_relative ", "require "];
    for prefix in &prefixes {
        if line.starts_with(prefix) {
            let rest = &line[prefix.len()..];
            let rest = rest.trim();
            // Extract string value
            if (rest.starts_with('\'') && rest.ends_with('\''))
                || (rest.starts_with('"') && rest.ends_with('"'))
            {
                return Some(&rest[1..rest.len() - 1]);
            }
        }
    }
    None
}
