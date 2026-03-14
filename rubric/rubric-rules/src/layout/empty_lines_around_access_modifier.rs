use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyLinesAroundAccessModifier;

const ACCESS_MODIFIERS: &[&str] = &[
    "private",
    "protected",
    "public",
    "private_class_method",
    "public_class_method",
];

fn is_access_modifier(line: &str) -> bool {
    let trimmed = line.trim();
    ACCESS_MODIFIERS.contains(&trimmed)
}

fn is_class_module_opener(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("class ") || trimmed == "class"
        || trimmed.starts_with("module ") || trimmed == "module"
}

impl Rule for EmptyLinesAroundAccessModifier {
    fn name(&self) -> &'static str {
        "Layout/EmptyLinesAroundAccessModifier"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 0..n {
            if !is_access_modifier(&lines[i]) {
                continue;
            }

            // Find the previous non-blank line index
            let prev_non_blank = (0..i).rev().find(|&j| !lines[j].trim().is_empty());

            // Find the next non-blank line index
            let next_non_blank = ((i + 1)..n).find(|&j| !lines[j].trim().is_empty());

            // Check blank line BEFORE: required unless previous non-blank is a class/module opener
            // or there is no previous non-blank line (modifier at top of class body)
            if let Some(prev_idx) = prev_non_blank {
                let prev_line = &lines[prev_idx];
                let is_opener = is_class_module_opener(prev_line);
                // If previous non-blank line is immediately before this line (no blank), flag it
                // unless previous line is a class/module opener
                if !is_opener && prev_idx + 1 == i {
                    let line_start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Add an empty line before `{}`.",
                            lines[i].trim()
                        ),
                        range: TextRange::new(line_start, line_start + lines[i].len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }

            // Check blank line AFTER: required unless next non-blank is `end` or doesn't exist
            if let Some(next_idx) = next_non_blank {
                let next_line = lines[next_idx].trim();
                let is_end = next_line == "end" || next_line.starts_with("end ");
                if !is_end && next_idx == i + 1 {
                    let line_start = ctx.line_start_offsets[i];
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Add an empty line after `{}`.",
                            lines[i].trim()
                        ),
                        range: TextRange::new(line_start, line_start + lines[i].len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
