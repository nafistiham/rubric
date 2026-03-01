use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceInsideBlockBraces;

impl Rule for SpaceInsideBlockBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideBlockBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let n = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut pos = 0;
            let mut in_string: Option<u8> = None;

            while pos < n {
                let b = bytes[pos];
                match in_string {
                    Some(_) if b == b'\\' => { pos += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; pos += 1; continue; }
                    Some(_) => { pos += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); pos += 1; continue; }
                    None if b == b'#' => break,
                    None => {}
                }

                // Detect `{` in block context (after method call) not followed by space
                if b == b'{' {
                    // Check what's after `{`
                    let next = if pos + 1 < n { bytes[pos + 1] } else { 0 };
                    // Determine if `{` is a block or a hash literal.
                    // A hash literal follows `=`, `,`, `(`, `[`, `{`, or appears at
                    // the start of a trimmed line. Skip those.
                    let prev_nonspace = {
                        let mut p = pos;
                        let mut found = 0u8;
                        while p > 0 {
                            p -= 1;
                            if bytes[p] != b' ' && bytes[p] != b'\t' {
                                found = bytes[p];
                                break;
                            }
                        }
                        found
                    };
                    let is_hash_context = matches!(
                        prev_nonspace,
                        b'=' | b',' | b'(' | b'[' | b'{' | 0
                    ) || pos == line.len() - line.trim_start().len();

                    if !is_hash_context && next != b' ' && next != b'\n' && next != b'}' {
                        // Missing space after `{`
                        let flag_pos = (line_start + pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space missing inside block braces after `{`.".into(),
                            range: TextRange::new(flag_pos, flag_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                if b == b'}' && pos > 0 {
                    let prev = bytes[pos - 1];
                    if prev != b' ' && prev != b'\n' && prev != b'{' {
                        let flag_pos = (line_start + pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Space missing inside block braces before `}`.".into(),
                            range: TextRange::new(flag_pos, flag_pos + 1),
                            severity: Severity::Warning,
                        });
                    }
                }

                pos += 1;
            }
        }

        diags
    }
}
