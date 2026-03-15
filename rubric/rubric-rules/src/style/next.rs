use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Next;

/// Returns true if the line (trimmed) looks like a loop/iterator block opener.
/// Matches patterns like: `.each do |x|`, `.map do |x|`, `times do`, etc.
fn is_loop_block_opener(trimmed: &str) -> bool {
    // Match method names that are loop-like iterators
    let loop_methods = [
        "each", "map", "select", "reject", "collect", "detect", "find",
        "find_all", "each_with_index", "each_with_object", "times", "upto",
        "downto", "each_line", "each_slice", "each_cons", "flat_map",
        "filter_map", "loop",
    ];

    // Must contain `do` as a word (followed by optional `|...|`)
    // Check that `do` appears as a word token
    if !contains_word(trimmed, "do") {
        return false;
    }

    for method in &loop_methods {
        // Pattern: `.method` or just `method` (for `loop do`)
        let dot_method = format!(".{}", method);
        if trimmed.contains(dot_method.as_str()) || trimmed == "loop do" || trimmed.starts_with("loop do ") {
            return true;
        }
    }

    false
}

/// Returns true if `word` appears as a standalone word in `s`.
fn contains_word(s: &str, word: &str) -> bool {
    let bytes = s.as_bytes();
    let wbytes = word.as_bytes();
    let mut i = 0;
    while i + wbytes.len() <= bytes.len() {
        if bytes[i..].starts_with(wbytes) {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            let after = i + wbytes.len();
            let after_ok =
                after >= bytes.len() || !bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_';
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Count leading spaces for indentation level.
fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

impl Rule for Next {
    fn name(&self) -> &'static str {
        "Style/Next"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let line = lines[i];
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            if is_loop_block_opener(trimmed) {
                let block_indent = indent_of(line);

                // Find the first non-empty line inside the block
                let mut j = i + 1;
                while j < n && lines[j].trim().is_empty() {
                    j += 1;
                }

                if j < n {
                    let inner = lines[j];
                    let inner_trimmed = inner.trim_start();
                    let inner_indent = indent_of(inner);

                    // The inner line must be indented more than the block opener
                    // and must start with `if ` (not `if!` or `unless`)
                    if inner_indent > block_indent
                        && (inner_trimmed.starts_with("if ") || inner_trimmed == "if")
                    {
                        // Now verify: after the matching `end` for this `if`, the very
                        // next non-empty line is `end` at the block's indentation level.
                        // We use a simple end-counting approach.
                        if let Some(if_end_line) = find_matching_end(lines, j, inner_indent) {
                            // Check that the line after if_end_line (skipping blanks)
                            // is `end` at block_indent
                            let mut k = if_end_line + 1;
                            while k < n && lines[k].trim().is_empty() {
                                k += 1;
                            }
                            if k < n {
                                let closer = lines[k];
                                let closer_trimmed = closer.trim();
                                let closer_indent = indent_of(closer);
                                if closer_trimmed == "end" && closer_indent == block_indent {
                                    // Flag the `if` line
                                    let line_start = ctx.line_start_offsets[j] as usize;
                                    let col = inner_indent; // offset of `if` in the line
                                    let start = (line_start + col) as u32;
                                    let end = start + 2; // len("if")
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message:
                                            "Use next to skip iteration instead of wrapping \
                                             iteration body in a conditional."
                                                .into(),
                                        range: TextRange::new(start, end),
                                        severity: Severity::Warning,
                                    });
                                    i = k + 1;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }

            i += 1;
        }

        diags
    }
}

/// Given that `if_line` is the index of the `if` line with indentation `if_indent`,
/// find the index of the matching `end` line.
/// Uses a simple counter: each keyword that opens a block increments, `end` decrements.
fn find_matching_end(lines: &[&str], if_line: usize, if_indent: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = if_line;
    while i < lines.len() {
        let t = lines[i].trim();
        // Keywords that open a new end-terminated block
        let opens = t == "do"
            || t.starts_with("if ")
            || t == "if"
            || t.starts_with("unless ")
            || t == "unless"
            || t.starts_with("while ")
            || t == "while"
            || t.starts_with("until ")
            || t == "until"
            || t.starts_with("for ")
            || t.starts_with("begin")
            || t.starts_with("def ")
            || t.starts_with("class ")
            || t.starts_with("module ")
            || t.starts_with("case ")
            || t == "case"
            || (t.ends_with("do") && !t.starts_with('#'))
            || (t.contains(" do |") && !t.starts_with('#'))
            || (t.contains(" do\t") && !t.starts_with('#'));

        if opens && i == if_line {
            depth += 1;
        } else if opens && i > if_line {
            depth += 1;
        }

        // If the `if` has an `else` or `elsif` branch, it wraps the full body
        // with an alternative — this is NOT the simple case the cop targets.
        if depth == 1
            && i > if_line
            && (t == "else" || t.starts_with("elsif ") || t == "elsif")
        {
            return None;
        }

        if t == "end" || t.starts_with("end ") || t.starts_with("end.") || t == "end;" {
            if depth <= 1 {
                // Check that this end is at the right indentation
                let end_indent = indent_of(lines[i]);
                if end_indent == if_indent {
                    return Some(i);
                }
            }
            depth -= 1;
        }

        i += 1;
    }
    None
}
