use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceBeforeBlockBraces;

impl Rule for SpaceBeforeBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeBlockBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let len = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < len {
                let b = bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Skip %r{...}, %q{...}, %Q{...}, %w{...}, %W{...}, %i{...}, %I{...}, %s{...}, %x{...}
                // When the percent-literal uses `{` as delimiter, the `}` closes it — not a block brace
                if b == b'%' && j + 1 < len && bytes[j + 1].is_ascii_alphabetic() {
                    j += 2;
                    if j < len {
                        let delim = bytes[j];
                        j += 1;
                        if delim == b'{' {
                            let mut depth = 1usize;
                            while j < len && depth > 0 {
                                match bytes[j] {
                                    b'\\' => { j += 2; }
                                    b'{' => { depth += 1; j += 1; }
                                    b'}' => { depth -= 1; j += 1; }
                                    _ => { j += 1; }
                                }
                            }
                        } else {
                            let close = match delim {
                                b'(' => b')', b'[' => b']', b'<' => b'>', _ => delim,
                            };
                            while j < len && bytes[j] != close {
                                if bytes[j] == b'\\' { j += 2; } else { j += 1; }
                            }
                            if j < len { j += 1; }
                        }
                    }
                    continue;
                }

                // Skip /regex/ literals
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || prev == 0
                    {
                        j += 1;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'/' => { j += 1; break; }
                                _ => { j += 1; }
                            }
                        }
                        continue;
                    }
                }

                // Check for word char immediately followed by `{`
                if b == b'{' && j > 0 {
                    let prev = bytes[j - 1];
                    if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' || prev == b']' {
                        let brace_pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space before block `{`.".into(),
                            range: TextRange::new(brace_pos, brace_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }
                j += 1;
            }
        }

        diags
    }
}
