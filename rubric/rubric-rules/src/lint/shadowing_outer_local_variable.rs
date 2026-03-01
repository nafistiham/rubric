use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct ShadowingOuterLocalVariable;

impl Rule for ShadowingOuterLocalVariable {
    fn name(&self) -> &'static str {
        "Lint/ShadowingOuterLocalVariable"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Collect outer local variables (assignments outside blocks)
        let mut outer_locals: HashSet<String> = HashSet::new();

        // Simple two-pass: first collect all non-block assignments
        let mut block_depth = 0usize;
        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') { continue; }

            if t.contains(" do") || t.contains(" do |") { block_depth += 1; }
            if t == "end" && block_depth > 0 { block_depth -= 1; }

            if block_depth == 0 {
                // Collect assignments `var = value`
                if let Some(eq_pos) = t.find(" = ") {
                    let lhs = t[..eq_pos].trim();
                    if !lhs.is_empty()
                        && lhs.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
                        && !lhs.contains('.')
                        && !lhs.contains('[') {
                        outer_locals.insert(lhs.to_string());
                    }
                }
            }
        }

        // Second pass: find block parameters that shadow outer locals
        block_depth = 0;
        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') { continue; }

            // Check for block params `|x|` or `|x, y|`
            if (t.contains(" do |") || t.contains(" do|") || t.starts_with("do |")) && !t.contains("||}") {
                // Extract params
                if let Some(pipe_open) = t.find('|') {
                    if let Some(pipe_close) = t[pipe_open + 1..].find('|') {
                        let params_str = &t[pipe_open + 1..pipe_open + 1 + pipe_close];
                        for param in params_str.split(',') {
                            let p = param.trim().trim_start_matches('*').trim_start_matches('&');
                            if !p.is_empty() && outer_locals.contains(p) {
                                let indent = line.len() - trimmed.len();
                                let line_start = ctx.line_start_offsets[i] as usize;
                                let pos = (line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Block parameter `{}` shadows outer local variable.",
                                        p
                                    ),
                                    range: TextRange::new(pos, pos + t.len() as u32),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
                block_depth += 1;
            } else if t.contains(" do") || t == "do" {
                block_depth += 1;
            }

            // Also handle `{ |x| }` style blocks
            if t.contains("{ |") {
                // pipe_open points to the opening `|` character itself
                if let Some(brace_pos) = t.find("{ |") {
                    let pipe_open = brace_pos + "{ |".len() - 1; // index of opening `|`
                    if let Some(pipe_close) = t[pipe_open + 1..].find('|') {
                        let params_str = &t[pipe_open + 1..pipe_open + 1 + pipe_close];
                        for param in params_str.split(',') {
                            let p = param.trim();
                            if !p.is_empty() && outer_locals.contains(p) {
                                let indent = line.len() - trimmed.len();
                                let line_start = ctx.line_start_offsets[i] as usize;
                                let pos = (line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Block parameter `{}` shadows outer local variable.",
                                        p
                                    ),
                                    range: TextRange::new(pos, pos + t.len() as u32),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }

            if t == "end" && block_depth > 0 { block_depth -= 1; }
        }

        diags
    }
}
