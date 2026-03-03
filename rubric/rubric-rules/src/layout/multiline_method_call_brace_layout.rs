use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodCallBraceLayout;

impl Rule for MultilineMethodCallBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallBraceLayout"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 1..n {
            let line = &lines[i];
            let trimmed = line.trim();
            // Closing `)` on same line as last argument in multiline call
            if trimmed.ends_with(')') && !trimmed.starts_with(')') && trimmed.len() > 1 {
                // Skip if parentheses are balanced on this line — the trailing `)`
                // closes a `(` opened on the same line, not a previous-line multiline call.
                let open_count = trimmed.bytes().filter(|&b| b == b'(').count();
                let close_count = trimmed.bytes().filter(|&b| b == b')').count();
                if open_count >= close_count {
                    continue;
                }

                // Check if previous lines have unclosed `(`
                let mut j = i;
                let mut is_multiline = false;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim_end();
                    // Only treat as multiline-new-line style when the opening `(`
                    // is the last meaningful character on its line (bare paren).
                    // `foo(arg1,\n    arg2)` is valid symmetrical style — the `(`
                    // has content after it so it doesn't count here.
                    if prev.ends_with('(') {
                        is_multiline = true;
                        break;
                    }
                    if prev.trim().is_empty() { break; }
                }

                if is_multiline {
                    let indent = line.len() - line.trim_start().len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Closing `)` of multiline method call should be on its own line.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
