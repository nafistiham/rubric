use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SelectByRegexp;

/// Patterns that indicate regexp usage inside a block.
const REGEXP_INDICATORS: &[&str] = &["=~", ".match?(", ".match("];

impl Rule for SelectByRegexp {
    fn name(&self) -> &'static str {
        "Style/SelectByRegexp"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let (method, replacement) = if contains_select_block(line) {
                ("select", "grep")
            } else if contains_reject_block(line) {
                ("reject", "grep_v")
            } else {
                continue;
            };

            // Check that the line also contains a regexp indicator
            if !has_regexp_indicator(line) {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let indent = line.len() - trimmed.len();
            let start = (line_start + indent) as u32;
            let end = (line_start + line.trim_end().len()) as u32;

            diags.push(Diagnostic {
                rule: self.name(),
                message: format!(
                    "Prefer `{}` over `{}` with a regexp.",
                    replacement, method
                ),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}

/// Returns true if the line contains `.select {` or `.select{`.
fn contains_select_block(line: &str) -> bool {
    line.contains(".select {") || line.contains(".select{")
}

/// Returns true if the line contains `.reject {` or `.reject{`.
fn contains_reject_block(line: &str) -> bool {
    line.contains(".reject {") || line.contains(".reject{")
}

/// Returns true if the line contains any regexp indicator pattern.
fn has_regexp_indicator(line: &str) -> bool {
    REGEXP_INDICATORS.iter().any(|pat| line.contains(pat))
}
