use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashEachMethods;

impl Rule for HashEachMethods {
    fn name(&self) -> &'static str {
        "Style/HashEachMethods"
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
                    b'.' => {
                        // Look for `.keys.each` or `.values.each` starting at this `.`
                        let rest = &line[j..];
                        if rest.starts_with(".keys.each") {
                            // Make sure `each` is not followed by `_key` or `_value`
                            let after = &rest[".keys.each".len()..];
                            if !after.starts_with('_') {
                                let start = (line_start + j) as u32;
                                let end = (line_start + j + ".keys.each".len()) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use `Hash#each_key` instead of `Hash#keys#each`."
                                        .to_string(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += ".keys.each".len();
                                continue;
                            }
                        } else if rest.starts_with(".values.each") {
                            let after = &rest[".values.each".len()..];
                            if !after.starts_with('_') {
                                let start = (line_start + j) as u32;
                                let end = (line_start + j + ".values.each".len()) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: "Use `Hash#each_value` instead of `Hash#values#each`."
                                        .to_string(),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                                j += ".values.each".len();
                                continue;
                            }
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
