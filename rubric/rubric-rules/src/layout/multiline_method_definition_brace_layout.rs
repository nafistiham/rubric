use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineMethodDefinitionBraceLayout;

impl Rule for MultilineMethodDefinitionBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodDefinitionBraceLayout"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 1..n {
            let line = &lines[i];
            let trimmed = line.trim();

            // A line ending with `)` that contains a parameter (has content before `)`)
            // and the previous lines show a `def` with parameters that spans multiple lines
            if trimmed.ends_with(')') && !trimmed.starts_with("def ") && trimmed.len() > 1 {
                let mut j = i;
                let mut is_multiline_def = false;
                while j > 0 {
                    j -= 1;
                    let prev = lines[j].trim();
                    if prev.starts_with("def ") && prev.contains('(') {
                        // Check if the def's parens are balanced on the same line.
                        // If balanced, the def is complete — don't flag anything after it.
                        let opens = prev.chars().filter(|&c| c == '(').count();
                        let closes = prev.chars().filter(|&c| c == ')').count();
                        if opens == closes {
                            // Single-line def — not a multiline definition
                            break;
                        }
                        // Symmetrical style (RuboCop default): only flag when the first
                        // parameter is on a NEW line (nothing after `(` on the def line).
                        // If the first param is on the def line, the closing `)` may also
                        // be on the last param line — that is symmetric and correct.
                        let paren_pos = prev.find('(').unwrap_or(prev.len() - 1);
                        let after_paren = prev[paren_pos + 1..].trim();
                        if after_paren.is_empty() {
                            // First param on new line → `)` on content line = asymmetric → flag
                            is_multiline_def = true;
                        }
                        // else: first param on def line → symmetric → don't flag
                        break;
                    }
                    if prev.is_empty() { break; }
                }

                if is_multiline_def {
                    let indent = line.len() - line.trim_start().len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Closing `)` of multiline method definition should be on its own line.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
