use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideStringInterpolation;

impl Rule for SpaceInsideStringInterpolation {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideStringInterpolation"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let bytes = line.as_bytes();
            let len = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut in_string = false;
            let mut j = 0;
            while j < len {
                let b = bytes[j];
                if !in_string {
                    if b == b'"' {
                        in_string = true;
                    }
                } else {
                    if b == b'"' {
                        in_string = false;
                    } else if b == b'\\' {
                        j += 1; // skip escape
                    } else if b == b'#' && j + 1 < len && bytes[j + 1] == b'{' {
                        // Start of interpolation: check for space after #{
                        let interp_start = j;
                        j += 2; // skip #{
                        if j < len && bytes[j] == b' ' {
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space after `#{` in string interpolation.".into(),
                                range: TextRange::new(
                                    (line_start + j) as u32,
                                    (line_start + j + 1) as u32,
                                ),
                                severity: Severity::Warning,
                            });
                        }
                        // Scan to find the matching `}` and check space before it
                        let mut depth = 1i32;
                        let interp_body_start = j;
                        while j < len && depth > 0 {
                            if bytes[j] == b'{' { depth += 1; }
                            else if bytes[j] == b'}' { depth -= 1; }
                            if depth > 0 { j += 1; }
                        }
                        // j now points at closing `}`
                        if depth == 0 && j > interp_body_start && bytes[j - 1] == b' ' {
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space before `}` in string interpolation.".into(),
                                range: TextRange::new(
                                    (line_start + j - 1) as u32,
                                    (line_start + j) as u32,
                                ),
                                severity: Severity::Warning,
                            });
                        }
                        let _ = interp_start;
                        continue;
                    }
                }
                j += 1;
            }
        }

        diags
    }
}
