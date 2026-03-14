use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct DuplicateElsifCondition;

/// Extract the condition after `if ` or `elsif `.
fn extract_condition<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix(keyword)?;
    // Strip trailing `then` or comment
    let cond = rest.trim_end();
    let cond = cond.strip_suffix(" then").unwrap_or(cond);
    let cond = if let Some(pos) = cond.find(" #") {
        cond[..pos].trim_end()
    } else {
        cond
    };
    Some(cond)
}

impl Rule for DuplicateElsifCondition {
    fn name(&self) -> &'static str {
        "Lint/DuplicateElsifCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Stack of condition lists for nested if chains.
        // Each element is a Vec of conditions seen so far in the current chain.
        let mut chain_stack: Vec<Vec<String>> = Vec::new();
        // Track `if` depth so we know when a chain ends.
        let mut if_depth: Vec<usize> = Vec::new(); // depth of `if` that started this chain
        let mut current_depth: usize = 0;

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Track if/end depth (approximate — doesn't handle one-liners perfectly)
            if trimmed.starts_with("end") {
                // Pop chain if we're closing an if at the right depth
                if let Some(&chain_depth) = if_depth.last() {
                    if current_depth == chain_depth {
                        chain_stack.pop();
                        if_depth.pop();
                    }
                }
                if current_depth > 0 {
                    current_depth -= 1;
                }
                continue;
            }

            if let Some(cond) = extract_condition(line, "if ") {
                current_depth += 1;
                let new_chain = vec![cond.to_string()];
                chain_stack.push(new_chain);
                if_depth.push(current_depth);
                continue;
            }

            if let Some(cond) = extract_condition(line, "elsif ") {
                if let Some(chain) = chain_stack.last_mut() {
                    let cond_str = cond.to_string();
                    if chain.contains(&cond_str) {
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let col = line.len() - trimmed.len();
                        let start = (line_start + col) as u32;
                        let end = (line_start + line.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "Duplicate `elsif` condition detected (`{}`).",
                                cond_str
                            ),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    } else {
                        chain.push(cond_str);
                    }
                }
                continue;
            }

            // Rough depth tracking for other openers
            if trimmed.starts_with("def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("module ")
                || trimmed.starts_with("do")
                || trimmed.starts_with("begin")
                || trimmed.starts_with("case ")
                || trimmed.starts_with("unless ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("until ")
                || trimmed.starts_with("for ")
            {
                current_depth += 1;
            }
        }

        diags
    }
}
