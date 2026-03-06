use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantReturn;

impl Rule for RedundantReturn {
    fn name(&self) -> &'static str {
        "Style/RedundantReturn"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim();
            // Find the start of a method definition
            if trimmed.starts_with("def ") || trimmed == "def" {
                let def_line = i;
                let mut depth = 1usize;
                i += 1;
                // Scan to find the matching `end`
                while i < lines.len() && depth > 0 {
                    let t = lines[i].trim();
                    if t.starts_with("def ") || t.starts_with("class ")
                       || t.starts_with("module ") || t.starts_with("if ")
                       || t.starts_with("unless ") || t.starts_with("while ")
                       || t.starts_with("until ") || t.starts_with("for ")
                       || t == "case" || t.starts_with("case ")
                       || t == "do" || t.ends_with(" do") || t.contains(" do |") || t.contains(" do|")
                       || t.starts_with("begin") {
                        depth += 1;
                    } else if t == "end" {
                        depth -= 1;
                        if depth == 0 {
                            // This `end` closes the `def` — find last content line before it
                            let mut j = i.saturating_sub(1);
                            while j > def_line {
                                let last = lines[j].trim();
                                if !last.is_empty() && !last.starts_with('#') {
                                    if last.starts_with("return") {
                                        let line_start = ctx.line_start_offsets[j];
                                        let return_end = line_start + "return".len() as u32;
                                        diags.push(Diagnostic {
                                            rule: self.name(),
                                            message: "Redundant `return` in last expression of method.".into(),
                                            range: TextRange::new(line_start, return_end),
                                            severity: Severity::Warning,
                                        });
                                    }
                                    break;
                                }
                                j -= 1;
                            }
                        }
                    }
                    i += 1;
                }
                continue;
            }
            i += 1;
        }
        diags
    }
}
