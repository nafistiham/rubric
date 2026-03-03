use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct ShadowingOuterLocalVariable;

fn is_method_def(t: &str) -> bool {
    t.starts_with("def ") || t.starts_with("def self.")
}

impl Rule for ShadowingOuterLocalVariable {
    fn name(&self) -> &'static str {
        "Lint/ShadowingOuterLocalVariable"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track scope per method. When we enter a `def`, we start fresh.
        // outer_locals: variables assigned at depth-0 within the current method.
        let mut outer_locals: HashSet<String> = HashSet::new();
        let mut block_depth = 0usize;
        let mut method_depth = 0usize;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') {
                continue;
            }

            // Detect method entry: reset scope
            if is_method_def(t) {
                outer_locals.clear();
                block_depth = 0;
                method_depth += 1;
                // Collect method params as outer locals too if inline
                // e.g. `def foo(key)` → `key` is outer local
                if let Some(paren_open) = t.find('(') {
                    if let Some(paren_close) = t[paren_open..].find(')') {
                        let params_str = &t[paren_open + 1..paren_open + paren_close];
                        for param in params_str.split(',') {
                            let p = param.trim().trim_start_matches('*').trim_start_matches('&');
                            // strip default values
                            let p = p.split('=').next().unwrap_or(p).trim();
                            if !p.is_empty()
                                && p.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
                            {
                                outer_locals.insert(p.to_string());
                            }
                        }
                    }
                }
                // Endless method `def foo = expr` — no `end` expected, pop immediately
                if t.contains(" = ") {
                    let after_name = if let Some(p) = t.find('(') {
                        let close = t[p..].find(')').map(|c| p + c + 1).unwrap_or(t.len());
                        &t[close..]
                    } else {
                        &t["def ".len()..]
                    };
                    // if ` = ` appears after the name (not `==` or `=>`)
                    let mut found_endless = false;
                    let ab = after_name.as_bytes();
                    let mut j = 0;
                    while j + 2 < ab.len() {
                        if ab[j] == b' ' && ab[j + 1] == b'=' && ab[j + 2] != b'=' && ab[j + 2] != b'>' {
                            found_endless = true;
                            break;
                        }
                        j += 1;
                    }
                    if found_endless {
                        method_depth = method_depth.saturating_sub(1);
                    }
                }
                continue;
            }

            // `end` closes a block or a method
            if t == "end" {
                if block_depth > 0 {
                    block_depth -= 1;
                } else if method_depth > 0 {
                    method_depth -= 1;
                    outer_locals.clear();
                }
                continue;
            }

            // Detect `do |params|` block entry
            if t.contains(" do |") || t.contains(" do|") || t.starts_with("do |") {
                if let Some(pipe_open) = t.find('|') {
                    if let Some(pipe_close) = t[pipe_open + 1..].find('|') {
                        let params_str = &t[pipe_open + 1..pipe_open + 1 + pipe_close];
                        for param in params_str.split(',') {
                            let p = param.trim().trim_start_matches('*').trim_start_matches('&');
                            let p = p.split(':').next().unwrap_or(p).trim(); // strip keyword syntax
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
                continue;
            }

            if t.contains(" do") || t == "do" {
                block_depth += 1;
                continue;
            }

            // Detect `{ |params| }` style single-line block
            if let Some(brace_pos) = t.find("{ |") {
                let pipe_open = brace_pos + 2; // index of `|`
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
                // Single-line brace block: check if it closes on same line
                // If `}` appears after `{`, don't change block_depth
                if !t[brace_pos + 1..].contains('}') {
                    block_depth += 1;
                }
                continue;
            }

            // Collect outer locals (depth-0 assignments within method)
            if block_depth == 0 {
                if let Some(eq_pos) = t.find(" = ") {
                    let lhs = t[..eq_pos].trim();
                    if !lhs.is_empty()
                        && lhs.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
                        && !lhs.contains('.')
                        && !lhs.contains('[')
                    {
                        outer_locals.insert(lhs.to_string());
                    }
                }
            }
        }

        diags
    }
}
