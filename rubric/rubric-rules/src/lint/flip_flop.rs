use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct FlipFlop;

impl Rule for FlipFlop {
    fn name(&self) -> &'static str {
        "Lint/FlipFlop"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `if ..(..).. ` with `..` or `...` inside an if condition context
            // Simple: detect `if` lines containing `..` surrounded by non-numeric expressions
            if trimmed.starts_with("if ") || trimmed.starts_with("elsif ") {
                // Look for `)..(` or `=~..) ..` patterns that suggest flip-flop
                // Simple: if condition contains `..(` or `)..` (regex flip-flop patterns)
                let cond_part = &trimmed[trimmed.find(' ').unwrap_or(0)..];
                if (cond_part.contains(")..(") || cond_part.contains(")...("))
                    && cond_part.contains("=~") {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Avoid the flip-flop operator; it is confusing and rarely needed.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
