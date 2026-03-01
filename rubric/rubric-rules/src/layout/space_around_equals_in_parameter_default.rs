use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAroundEqualsInParameterDefault;

impl Rule for SpaceAroundEqualsInParameterDefault {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundEqualsInParameterDefault"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Only check lines with `def ` that have `(`
            if !trimmed.starts_with("def ") {
                continue;
            }

            let paren_start = match line.find('(') {
                Some(p) => p,
                None => continue,
            };

            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut pos = paren_start + 1;
            let mut depth = 1usize;

            while pos < n && depth > 0 {
                match bytes[pos] {
                    b'(' => { depth += 1; pos += 1; }
                    b')' => { depth -= 1; pos += 1; }
                    b'=' => {
                        // Check if it's `=` not `==`, `=>`, `!=`, `<=`, `>=`
                        let prev = if pos > 0 { bytes[pos - 1] } else { 0 };
                        let next = if pos + 1 < n { bytes[pos + 1] } else { 0 };
                        let missing_space_before = prev != b' ' && prev != b'!' && prev != b'<' && prev != b'>';
                        let missing_space_after = next != b' ' && next != b'=' && next != b'>';

                        if missing_space_before || missing_space_after {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let eq_abs = (line_start + pos) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Surrounding space missing for operator `=` in parameter default.".into(),
                                range: TextRange::new(eq_abs, eq_abs + 1),
                                severity: Severity::Warning,
                            });
                        }
                        pos += 1;
                    }
                    _ => { pos += 1; }
                }
            }
        }

        diags
    }
}
