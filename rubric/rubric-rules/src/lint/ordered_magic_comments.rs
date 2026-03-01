use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct OrderedMagicComments;

impl Rule for OrderedMagicComments {
    fn name(&self) -> &'static str {
        "Lint/OrderedMagicComments"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Find magic comments at top of file
        let mut frozen_line: Option<usize> = None;
        let mut encoding_line: Option<usize> = None;

        for i in 0..n {
            let trimmed = lines[i].trim();
            if !trimmed.starts_with('#') {
                break; // Stop at first non-comment
            }
            if trimmed.contains("frozen_string_literal:") {
                frozen_line = Some(i);
            }
            if trimmed.contains("encoding:") {
                encoding_line = Some(i);
            }
        }

        // encoding should come before frozen_string_literal
        if let (Some(frozen), Some(encoding)) = (frozen_line, encoding_line) {
            if frozen < encoding {
                let line_start = ctx.line_start_offsets[encoding] as u32;
                let len = lines[encoding].len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Magic comments should be ordered: `encoding` before `frozen_string_literal`.".into(),
                    range: TextRange::new(line_start, line_start + len),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
