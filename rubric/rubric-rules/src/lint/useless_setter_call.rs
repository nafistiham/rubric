use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UselessSetterCall;

impl Rule for UselessSetterCall {
    fn name(&self) -> &'static str {
        "Lint/UselessSetterCall"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("def ") {
                let def_line = i;
                let mut depth = 1usize;
                let mut j = i + 1;
                while j < n && depth > 0 {
                    let t = lines[j].trim();
                    if t.starts_with("def ") || t.starts_with("if ") || t.starts_with("unless ")
                        || t.starts_with("while ") || t.starts_with("until ")
                        || t.contains(" do") || t == "do" || t.starts_with("begin") {
                        depth += 1;
                    }
                    if t == "end" { depth -= 1; }
                    j += 1;
                }
                let def_end = j;

                // Find last non-empty, non-comment line before `end`
                let mut last_content = def_end.saturating_sub(2); // skip the `end` line
                while last_content > def_line {
                    let t = lines[last_content].trim();
                    if !t.is_empty() && !t.starts_with('#') {
                        break;
                    }
                    if last_content == 0 { break; }
                    last_content -= 1;
                }

                if last_content > def_line {
                    let last_line = lines[last_content].trim();
                    // Check if it's `self.something =`
                    if last_line.starts_with("self.") && last_line.contains(" = ") {
                        let indent = lines[last_content].len() - lines[last_content].trim_start().len();
                        let line_start = ctx.line_start_offsets[last_content] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Useless setter call at end of method; return value is discarded.".into(),
                            range: TextRange::new(pos, pos + last_line.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }

                i = def_end;
                continue;
            }
            i += 1;
        }

        diags
    }
}
