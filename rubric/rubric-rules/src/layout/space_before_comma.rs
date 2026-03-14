use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceBeforeComma;

impl Rule for SpaceBeforeComma {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeComma"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                if let Some(delim) = in_string {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        c if c == delim => {
                            in_string = None;
                        }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                // Outside a string
                match b {
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                    }
                    b'#' => break, // inline comment — stop
                    b',' => {
                        // Check if there's whitespace immediately before this comma
                        if j > 0 && (bytes[j - 1] == b' ' || bytes[j - 1] == b'\t') {
                            // Find the start of the whitespace run
                            let mut ws_start = j - 1;
                            while ws_start > 0
                                && (bytes[ws_start - 1] == b' ' || bytes[ws_start - 1] == b'\t')
                            {
                                ws_start -= 1;
                            }
                            let start = (line_start + ws_start) as u32;
                            let end = (line_start + j + 1) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space found before comma.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
        }

        diags
    }
}
