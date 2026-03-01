use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyBlock;

impl Rule for EmptyBlock {
    fn name(&self) -> &'static str {
        "Lint/EmptyBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect `{ }` — empty block with just whitespace
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut j = 0;

            while j < len {
                if bytes[j] == b'{' {
                    let open_pos = j;
                    j += 1;
                    // Skip spaces
                    while j < len && bytes[j] == b' ' { j += 1; }
                    // Check if we hit `}`
                    if j < len && bytes[j] == b'}' {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Empty block detected.".into(),
                            range: TextRange::new((line_start + open_pos) as u32, (line_start + j + 1) as u32),
                            severity: Severity::Warning,
                        });
                    }
                    continue;
                }
                j += 1;
            }

            // Also detect `do\nend` (empty do..end block)
            // Check if line ends with `do` and next non-blank line is `end`
            let trimmed_end = line.trim_end();
            if trimmed_end.ends_with(" do") || trimmed_end == "do" {
                if i + 1 < n && lines[i + 1].trim() == "end" {
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let do_pos = trimmed_end.len() - 2;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Empty `do..end` block detected.".into(),
                        range: TextRange::new((line_start + do_pos) as u32, (line_start + do_pos + 2) as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
