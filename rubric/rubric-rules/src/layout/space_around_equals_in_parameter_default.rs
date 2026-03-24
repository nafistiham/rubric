use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

/// `no_space: false` (default) → require spaces around `=` (`def foo(x = 1)`).
/// `no_space: true`  → require no spaces around `=` (`def foo(x=1)`).
pub struct SpaceAroundEqualsInParameterDefault {
    pub no_space: bool,
}

impl Default for SpaceAroundEqualsInParameterDefault {
    fn default() -> Self {
        Self { no_space: false }
    }
}

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
            let mut in_string: Option<u8> = None;

            while pos < n && depth > 0 {
                // ── String literal tracking ──────────────────────────────────
                if let Some(delim) = in_string {
                    if bytes[pos] == b'\\' { pos += 2; continue; }
                    if bytes[pos] == delim { in_string = None; }
                    pos += 1;
                    continue;
                }
                if bytes[pos] == b'"' || bytes[pos] == b'\'' {
                    in_string = Some(bytes[pos]);
                    pos += 1;
                    continue;
                }

                match bytes[pos] {
                    b'(' => { depth += 1; pos += 1; }
                    b')' => { depth -= 1; pos += 1; }
                    b'=' => {
                        // Check if it's `=` not `==`, `=>`, `!=`, `<=`, `>=`
                        let prev = if pos > 0 { bytes[pos - 1] } else { 0 };
                        let next = if pos + 1 < n { bytes[pos + 1] } else { 0 };
                        let missing_space_before = prev != b' ' && prev != b'!' && prev != b'<' && prev != b'>';
                        let missing_space_after = next != b' ' && next != b'=' && next != b'>';

                        let violation = if self.no_space {
                            // no_space style: flag when spaces ARE present
                            !missing_space_before || !missing_space_after
                        } else {
                            // space style (default): flag when spaces are missing
                            missing_space_before || missing_space_after
                        };
                        if violation {
                            let line_start = ctx.line_start_offsets[i] as usize;
                            let eq_abs = (line_start + pos) as u32;
                            let msg = if self.no_space {
                                "Surrounding space detected for operator `=` in parameter default."
                            } else {
                                "Surrounding space missing for operator `=` in parameter default."
                            };
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: msg.into(),
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
