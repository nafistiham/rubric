use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineArrayBraceLayout;

impl Rule for MultilineArrayBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineArrayBraceLayout"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track whether we've seen a multiline array (started with `[` at end of a line
        // or `= [` with content that spans multiple lines).
        // Simple: detect `[...,` content followed by `]` on a line with non-`]` content
        for i in 1..n {
            let line = &lines[i];
            let trimmed = line.trim();
            // Look for `]` at the end of a line that also has other content
            // and the previous line(s) indicate this is a multiline array
            if trimmed.ends_with(']') && !trimmed.starts_with('[') && trimmed.len() > 1 {
                // Check if it looks like the closing bracket is on same line as content
                // (the `]` is not alone on its line)
                let closing_bracket_alone = trimmed == "]";
                if !closing_bracket_alone {
                    // Check if there's a matching `[` on a previous line (multiline)
                    let mut is_multiline = false;
                    let mut j = i;
                    while j > 0 {
                        j -= 1;
                        let prev = lines[j].trim();
                        if prev.trim_end().ends_with('[') {
                            is_multiline = true;
                            break;
                        }
                        if prev.is_empty() { break; }
                    }

                    if is_multiline {
                        let indent = line.len() - line.trim_start().len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Closing `]` of multiline array should be on its own line.".into(),
                            range: TextRange::new(pos, pos + trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}
