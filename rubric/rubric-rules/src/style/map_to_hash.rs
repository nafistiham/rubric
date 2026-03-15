use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MapToHash;

/// Returns true if the line is a full-line comment (trimmed starts with `#`).
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check whether `}.to_h` or `end.to_h` appears at the end of a trimmed line.
fn ends_with_to_h(trimmed: &str) -> bool {
    trimmed.ends_with("}.to_h") || trimmed.ends_with("end.to_h")
}

/// Check whether `.map {` or `.map do` appears in the line, indicating the
/// start of a `.map` block.
fn has_map_block_start(line: &str) -> bool {
    line.contains(".map {") || line.contains(".map do")
}

impl Rule for MapToHash {
    fn name(&self) -> &'static str {
        "Style/MapToHash"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            if is_comment_line(line) {
                continue;
            }

            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Single-line: `.map { ... }.to_h` on one line
            if has_map_block_start(line) && ends_with_to_h(trimmed) {
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Prefer to_h over map.to_h.".into(),
                    range: TextRange::new(pos, pos + line.trim_end().len() as u32),
                    severity: Severity::Warning,
                });
                continue;
            }

            // Multi-line: line ends with `}.to_h` or `end.to_h` and `.map` appears
            // within 5 preceding lines
            if ends_with_to_h(trimmed) {
                let look_back = i.saturating_sub(5);
                let preceding_has_map = ctx.lines[look_back..i]
                    .iter()
                    .any(|l| has_map_block_start(l));
                if preceding_has_map {
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Prefer to_h over map.to_h.".into(),
                        range: TextRange::new(pos, pos + line.trim_end().len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
