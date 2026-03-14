use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct PerlBackrefs;

impl Rule for PerlBackrefs {
    fn name(&self) -> &'static str {
        "Style/PerlBackrefs"
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

                match b {
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                    }
                    b'#' => break, // inline comment — stop
                    b'$' => {
                        // Check if next char is a digit 1-9
                        if j + 1 < n && bytes[j + 1].is_ascii_digit() && bytes[j + 1] != b'0' {
                            let start = (line_start + j) as u32;
                            let end = (line_start + j + 2) as u32;
                            let var_name = format!("${}", bytes[j + 1] as char);
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Prefer `Regexp.last_match({})` over `{}`.",
                                    bytes[j + 1] as char,
                                    var_name
                                ),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                            j += 2;
                            continue;
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
