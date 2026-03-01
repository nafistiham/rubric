use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct LeadingCommentSpace;

impl Rule for LeadingCommentSpace {
    fn name(&self) -> &'static str {
        "Layout/LeadingCommentSpace"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Only check lines where the first non-space character is `#`
            if !trimmed.starts_with('#') {
                continue;
            }

            let bytes = trimmed.as_bytes();
            // Just `#` alone — OK
            if bytes.len() == 1 {
                continue;
            }

            // Skip shebangs `#!`
            if bytes[1] == b'!' {
                continue;
            }

            // Skip encoding/magic comments like `# encoding:`, `# frozen_string_literal:`
            let after_hash = &trimmed[1..];
            if after_hash.starts_with(" encoding:") || after_hash.starts_with(" frozen_string_literal:") {
                continue;
            }

            // Flag if char after `#` is not a space
            if bytes[1] != b' ' {
                // Find offset of `#` in original line
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let hash_pos = (line_start + indent) as u32;

                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Missing space after `#` in comment.".into(),
                    range: TextRange::new(hash_pos, hash_pos + 1),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Replace `#` with `# ` (insert space after hash)
        Some(Fix {
            edits: vec![TextEdit {
                range: TextRange::new(diag.range.start, diag.range.end),
                replacement: "# ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}
