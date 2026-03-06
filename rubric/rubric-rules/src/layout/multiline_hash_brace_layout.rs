use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineHashBraceLayout;

impl Rule for MultilineHashBraceLayout {
    fn name(&self) -> &'static str {
        "Layout/MultilineHashBraceLayout"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        for i in 1..n {
            let line = &lines[i];
            let trimmed = line.trim();
            // Look for `}` at end of a line with content (multiline hash closing on same line).
            // Skip comment lines — `# }` in doc comments must not be treated as code.
            if trimmed.ends_with('}') && !trimmed.starts_with('{') && !trimmed.starts_with('#') && trimmed.len() > 1 {
                let closing_alone = trimmed == "}";
                if !closing_alone {
                    // If the line has as many (or more) `{` as `}`, the closing `}` is
                    // matched by a `{` on the same line — single-line block, not a multiline
                    // hash closer.  e.g. `format.all { super(**) }`, `-> { expr }`,
                    // `@h = {}`, `#{interpolation}`.
                    let open_count = trimmed.bytes().filter(|&b| b == b'{').count();
                    let close_count = trimmed.bytes().filter(|&b| b == b'}').count();
                    if open_count >= close_count {
                        continue;
                    }

                    // Skip lines starting with `\` — these are inside multiline regex or
                    // string literals (e.g. `\}\}` in a `/pattern/x` regex body).
                    if trimmed.starts_with('\\') {
                        continue;
                    }

                    // Check if there's an opening `{` on a previous line
                    let mut j = i;
                    let mut is_multiline = false;
                    while j > 0 {
                        j -= 1;
                        let prev = lines[j].trim();
                        // Skip comment lines — don't let `# {` trigger is_multiline
                        if prev.starts_with('#') { continue; }
                        if prev.trim_end().ends_with('{') {
                            is_multiline = true;
                            break;
                        }
                        if prev.is_empty() { break; }
                    }

                    if is_multiline {
                        let indent = line.len() - line.trim_start().len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Closing `}` of multiline hash should be on its own line.".into(),
                            range: TextRange::new(pos, pos + trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        diags
    }
}
