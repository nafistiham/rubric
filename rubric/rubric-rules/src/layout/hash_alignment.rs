use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashAlignment;

impl Rule for HashAlignment {
    fn name(&self) -> &'static str {
        "Layout/HashAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip pure comment lines
            if line.trim_start().starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < len {
                let b = bytes[j];
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
                    None if b == b'#' => break, // inline comment — stop
                    None => {}
                }

                // Detect `=>`
                if j + 1 < len && bytes[j] == b'=' && bytes[j + 1] == b'>' {
                    // Count spaces immediately before `=>`
                    let mut k = j as isize - 1;
                    let mut space_count = 0usize;
                    while k >= 0 && bytes[k as usize] == b' ' {
                        space_count += 1;
                        k -= 1;
                    }

                    // Key style: exactly one space before `=>` is correct.
                    // Extra spaces (padding to align in table style) are a violation.
                    if space_count > 1 {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "Hash rocket `=>` has {} spaces before it; expected 1.",
                                space_count
                            ),
                            range: TextRange::new(pos, pos + 2),
                            severity: Severity::Warning,
                        });
                    }

                    j += 2; // skip `=>`
                    continue;
                }

                j += 1;
            }
        }

        diags
    }
}
