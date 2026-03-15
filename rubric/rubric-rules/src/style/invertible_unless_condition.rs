use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct InvertibleUnlessCondition;

/// Returns true if `!` at the given position in `s` is a logical negation
/// (not `!=` or `!==`).
fn is_logical_not(s: &str, bang_pos: usize) -> bool {
    let after = &s[bang_pos + 1..];
    !after.starts_with('=')
}

impl Rule for InvertibleUnlessCondition {
    fn name(&self) -> &'static str {
        "Style/InvertibleUnlessCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Skip full-line comments.
            if trimmed.starts_with('#') {
                continue;
            }

            // Block-form: line starts with `unless !`
            if trimmed.starts_with("unless ") {
                let after_unless = trimmed["unless ".len()..].trim_start();
                if after_unless.starts_with('!') && is_logical_not(after_unless, 0) {
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Avoid using unless with a negated condition. Prefer if.".into(),
                        range: TextRange::new(pos, pos + "unless".len() as u32),
                        severity: Severity::Warning,
                    });
                    continue;
                }
            }

            // Modifier-form: ` unless !` appears inside the line
            let mut search_start = 0;
            while let Some(rel) = trimmed[search_start..].find(" unless !") {
                let abs_pos = search_start + rel;
                // The `!` is 9 chars after the space: " unless !" = 9 chars
                let bang_pos_in_trimmed = abs_pos + " unless !".len() - 1;
                if is_logical_not(trimmed, bang_pos_in_trimmed) {
                    let pos = (line_start + indent + abs_pos + 1) as u32; // +1: skip leading space
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Avoid using unless with a negated condition. Prefer if.".into(),
                        range: TextRange::new(pos, pos + "unless".len() as u32),
                        severity: Severity::Warning,
                    });
                }
                search_start = abs_pos + 1;
            }
        }

        diags
    }
}
