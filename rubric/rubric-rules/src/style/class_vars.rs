use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClassVars;

impl Rule for ClassVars {
    fn name(&self) -> &'static str {
        "Style/ClassVars"
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
                    b'@' => {
                        // Check for `@@word`
                        if j + 2 < n
                            && bytes[j + 1] == b'@'
                            && (bytes[j + 2].is_ascii_alphabetic() || bytes[j + 2] == b'_')
                        {
                            // Find the end of the variable name
                            let var_start = j;
                            let mut k = j + 2;
                            while k < n
                                && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                            {
                                k += 1;
                            }
                            let var_name =
                                std::str::from_utf8(&bytes[var_start..k]).unwrap_or("@@var");
                            let start = (line_start + var_start) as u32;
                            let end = (line_start + k) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: format!(
                                    "Replace class var `{}` with a class instance variable.",
                                    var_name
                                ),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                            j = k;
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
