use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MultilineBlockChain;

impl Rule for MultilineBlockChain {
    fn name(&self) -> &'static str {
        "Style/MultilineBlockChain"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Only consider lines that start with `}.` or `end.` — the close of
            // one block immediately chained into another method call.
            let is_brace_chain = trimmed.starts_with("}.");
            let is_end_chain   = trimmed.starts_with("end.");

            if !is_brace_chain && !is_end_chain {
                continue;
            }

            // For `}.` chains, verify the matching `{` opened a block, not a hash literal.
            // Heuristic: scan backward to find the matching `{`. If that `{` is the
            // first non-space character on its line (i.e., `{` starts the line as a
            // hash literal), skip it.
            if is_brace_chain {
                let indent_width = line.len() - trimmed.len();
                let mut depth: isize = 1;
                let mut is_brace_block = false;
                let mut j = i;
                // Start scanning backward from this line (the `}` may be on this line too)
                'outer: loop {
                    if j == 0 { break; }
                    j -= 1;
                    let prev_line = &ctx.lines[j];
                    let prev_bytes = prev_line.as_bytes();
                    // Scan from right to left looking for `{` and `}` (simple, ignoring strings)
                    let mut k = prev_bytes.len();
                    while k > 0 {
                        k -= 1;
                        let c = prev_bytes[k];
                        if c == b'}' {
                            depth += 1;
                        } else if c == b'{' {
                            depth -= 1;
                            if depth == 0 {
                                // Found the matching `{`. Check indentation of its line.
                                let open_indent = prev_line.len() - prev_line.trim_start().len();
                                // If `{` is the first non-space char on its line, it's a
                                // hash literal (standalone `{` or `{ key: val }`).
                                let first_nonspace = k == open_indent;
                                // If the opening `{` line is at same indent as our `}.` line,
                                // AND the `{` is the first non-space char, it's a hash literal.
                                if first_nonspace && open_indent == indent_width {
                                    is_brace_block = false;
                                } else {
                                    is_brace_block = true;
                                }
                                break 'outer;
                            }
                        }
                    }
                }
                if !is_brace_block {
                    continue;
                }
            }

            // For `end.` chains, verify the matching `end` closes a `do...end` block,
            // not a control-flow expression (if/unless/case/begin/while/until/for).
            // Heuristic: scan backward, tracking `end` depth, and check what keyword
            // opens the block at depth 0. If it's `do`, flag it; otherwise skip.
            if is_end_chain {
                let indent_width = line.len() - trimmed.len();
                let mut depth: isize = 1; // we need to find the matching opener
                let mut is_do_block = false;
                let mut j = i;
                while j > 0 {
                    j -= 1;
                    let prev = ctx.lines[j].trim_start();
                    // Count `end` keywords to track nesting
                    if prev == "end" || prev.starts_with("end ") || prev.starts_with("end.") || prev.starts_with("end;") {
                        depth += 1;
                    }
                    // Count openers: do, if, unless, case, begin, while, until, for, def, class, module
                    let opens_do = prev.contains(" do") || prev.ends_with(" do")
                        || prev.contains(" do |") || prev.contains(" do|");
                    let opens_control = prev.starts_with("if ") || prev.starts_with("unless ")
                        || prev.starts_with("case ") || prev.starts_with("begin")
                        || prev.starts_with("while ") || prev.starts_with("until ")
                        || prev.starts_with("for ") || prev.starts_with("def ")
                        || prev.starts_with("class ") || prev.starts_with("module ");
                    if opens_do || opens_control {
                        depth -= 1;
                        if depth == 0 {
                            // Check indentation matches our `end.` line
                            let prev_indent = ctx.lines[j].len() - prev.len();
                            if prev_indent == indent_width {
                                is_do_block = opens_do && !opens_control;
                            }
                            break;
                        }
                    }
                }
                if !is_do_block {
                    continue;
                }
            }

            // Extract the method name after the dot.
            let dot_pos = match trimmed.find('.') {
                Some(p) => p,
                None    => continue,
            };
            let after_dot = &trimmed[dot_pos + 1..];
            // Method names may end with `?` or `!`
            let name_end = after_dot
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '?' && c != '!')
                .unwrap_or(after_dot.len());
            if name_end == 0 {
                continue;
            }
            let after_method = after_dot[name_end..].trim_end();

            // RuboCop flags only when the chained method *itself* directly receives
            // a new block — not when a block appears inside an argument to it.
            //
            // Genuine (flag):   }.map { |x|        — block right after method name
            //                   end.select do |x|  — block right after method name
            //                   }.select(n) { |x|  — block after method args
            // NOT genuine:      }.to change { x }  — `change { x }` is an *argument*
            //                   end.join(",")      — no block at all
            let rest = after_method.trim_start();
            let opens_new_block = rest.starts_with('{')
                || rest.starts_with("do ")
                || rest.starts_with("do|")
                || rest == "do"
                || (rest.starts_with('(') && (rest.ends_with('{')
                    || rest.ends_with('|')
                    || rest.ends_with("do")));

            if !opens_new_block {
                continue;
            }

            let indent     = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;
            let start      = (line_start + indent) as u32;
            let end_off    = (line_start + line.trim_end().len()) as u32;

            diags.push(Diagnostic {
                rule:     self.name(),
                message:  "Avoid multi-line chains of blocks.".into(),
                range:    TextRange::new(start, end_off),
                severity: Severity::Warning,
            });
        }

        diags
    }
}
