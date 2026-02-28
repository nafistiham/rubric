use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrailingCommaInArguments;

impl Rule for TrailingCommaInArguments {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInArguments"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_end();
            let bytes = trimmed.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Single-line check: look for , followed by optional whitespace then )
            let mut j = 0;
            while j < bytes.len() {
                if bytes[j] == b',' {
                    let rest = &bytes[j + 1..];
                    let spaces: usize = rest
                        .iter()
                        .take_while(|&&b| b == b' ' || b == b'\t')
                        .count();
                    if rest.get(spaces).copied() == Some(b')') {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Trailing comma in argument list.".into(),
                            range: TextRange::new(pos, pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
                j += 1;
            }

            // Multi-line check: line ends with `,` and next non-empty line is just `)`
            if trimmed.ends_with(',') {
                let next_non_empty = ctx.lines[i + 1..].iter().find(|l| !l.trim().is_empty());
                if next_non_empty.map(|l| l.trim() == ")").unwrap_or(false) {
                    let comma_pos = (line_start + trimmed.len() - 1) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Trailing comma in argument list.".into(),
                        range: TextRange::new(comma_pos, comma_pos + 1),
                        severity: Severity::Warning,
                    });
                }
            }
        }
        diags
    }
}
