use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NegatedIf;

impl Rule for NegatedIf {
    fn name(&self) -> &'static str {
        "Style/NegatedIf"
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

            // ── Block-form: `if !condition` where `if` is the first token ──
            if trimmed.starts_with("if ") {
                let after_if = trimmed["if ".len()..].trim_start();
                if after_if.starts_with('!') {
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `unless` instead of `if !`.".into(),
                        range: TextRange::new(pos, pos + 2),
                        severity: Severity::Warning,
                    });
                    continue; // block-form found; no need to check modifier-form too
                }
            }

            // ── Modifier-form: `expression if !condition` ──────────────────
            // Search for ` if !` anywhere in the trimmed line.  ` if !` (with
            // a leading space) never matches the block-form case (which starts
            // with `if`, no leading space in the trimmed text).
            if let Some(rel_pos) = trimmed.find(" if !") {
                let pos = (line_start + indent + rel_pos + 1) as u32; // +1: skip leading space
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Use `unless` instead of `if !`.".into(),
                    range: TextRange::new(pos, pos + 2),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
