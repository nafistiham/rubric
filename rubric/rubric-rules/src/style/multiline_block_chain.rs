use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineBlockChain;

/// Methods chained on a block close that are considered idiomatic/acceptable.
/// These are exempt from the MultilineBlockChain cop.
const EXEMPT_METHODS: &[&str] = &["freeze", "to_s", "inspect", "to_a", "to_h"];

/// Given a trimmed line that starts with `}.` or `end.`, return the method
/// name immediately following the dot. Returns `None` if there is no dot.
fn method_after_dot(trimmed: &str) -> Option<&str> {
    // Find the first '.' in the trimmed line
    let dot_pos = trimmed.find('.')?;
    let after_dot = &trimmed[dot_pos + 1..];
    // Collect the method name (alphanumeric + underscore)
    let end = after_dot
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(after_dot.len());
    if end == 0 {
        return None;
    }
    Some(&after_dot[..end])
}

impl Rule for MultilineBlockChain {
    fn name(&self) -> &'static str {
        "Style/MultilineBlockChain"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Check for }.method or end.method patterns
            let is_brace_chain = trimmed.starts_with("}.");
            let is_end_chain = trimmed.starts_with("end.");

            if !is_brace_chain && !is_end_chain {
                continue;
            }

            // Get the method name after the dot
            let method_name = match method_after_dot(trimmed) {
                Some(m) => m,
                None => continue,
            };

            // Skip exempt methods (idiomatic conversions/freeze)
            if EXEMPT_METHODS.contains(&method_name) {
                continue;
            }

            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let start = (line_start + indent) as u32;
            let end = (line_start + line.trim_end().len()) as u32;

            diags.push(Diagnostic {
                rule: self.name(),
                message: "Avoid multi-line chains of blocks.".into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}
