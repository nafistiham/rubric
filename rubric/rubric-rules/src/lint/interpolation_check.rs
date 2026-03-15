use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct InterpolationCheck;

impl Rule for InterpolationCheck {
    fn name(&self) -> &'static str {
        "Lint/InterpolationCheck"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[line_idx] as usize;

            let mut i = 0;
            while i < bytes.len() {
                // Skip escape sequences
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }

                // Look for a single-quoted string opening
                if bytes[i] == b'\'' {
                    let open_pos = i;
                    i += 1;

                    // Scan inside the single-quoted string
                    while i < bytes.len() {
                        // Handle escaped characters inside single-quoted string
                        if bytes[i] == b'\\' && i + 1 < bytes.len() {
                            i += 2;
                            continue;
                        }
                        // Closing quote
                        if bytes[i] == b'\'' {
                            i += 1;
                            break;
                        }
                        // Detect `#{` inside single-quoted string
                        if bytes[i] == b'#' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                            let start = (line_start + open_pos) as u32;
                            let end = start + 1;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Interpolation in single-quoted string detected.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                            // Skip past the `#{...}` — find the matching `}`
                            i += 2; // skip `#{`
                            let mut depth = 1u32;
                            while i < bytes.len() && depth > 0 {
                                if bytes[i] == b'{' {
                                    depth += 1;
                                } else if bytes[i] == b'}' {
                                    depth -= 1;
                                }
                                i += 1;
                            }
                            continue;
                        }
                        i += 1;
                    }
                } else if bytes[i] == b'"' {
                    // Skip double-quoted strings (including interpolation inside them)
                    i += 1;
                    let mut depth = 0u32;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' && i + 1 < bytes.len() {
                            i += 2;
                            continue;
                        }
                        if bytes[i] == b'"' && depth == 0 {
                            i += 1;
                            break;
                        }
                        if bytes[i] == b'#' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                            depth += 1;
                            i += 2;
                            continue;
                        }
                        if depth > 0 && bytes[i] == b'}' {
                            depth -= 1;
                        }
                        i += 1;
                    }
                } else if bytes[i] == b'#' {
                    // Rest of line is a comment — stop scanning
                    break;
                } else {
                    i += 1;
                }
            }
        }

        diags
    }
}
