use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NegatedIfElseCondition;

const MESSAGE: &str = "Invert if-else to remove the negation from the condition.";

/// Returns true if the trimmed line starts with `if !`.
fn is_negated_if(trimmed: &str) -> bool {
    trimmed.starts_with("if !") || trimmed.starts_with("if(!") || trimmed.starts_with("if !")
}

/// Returns true if the condition after `if !` is compound (has top-level `&&` or `||`).
/// Rubocop only flags simple `if !expr` with else, not compound `if !a && b` where
/// inverting would require de Morgan's law and may change semantics.
fn is_compound_negated_condition(trimmed: &str) -> bool {
    let cond = if let Some(rest) = trimmed.strip_prefix("if(") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("if ") {
        rest
    } else {
        return false;
    };
    if !cond.starts_with('!') {
        return false;
    }
    let bytes = cond.as_bytes();
    let mut depth: i32 = 0;
    let mut j = 0;
    while j < bytes.len() {
        match bytes[j] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => { if depth > 0 { depth -= 1; } }
            b'#' if depth == 0 => break,
            b'&' if depth == 0 && bytes.get(j + 1).copied() == Some(b'&') => return true,
            b'|' if depth == 0 && bytes.get(j + 1).copied() == Some(b'|') => return true,
            _ => {}
        }
        j += 1;
    }
    false
}

/// Returns true if the trimmed line is `else` (standalone).
fn is_else(trimmed: &str) -> bool {
    trimmed == "else"
}

/// Returns true if the trimmed line starts with `end` (end keyword).
fn is_end(trimmed: &str) -> bool {
    trimmed == "end"
        || trimmed.starts_with("end ")
        || trimmed.starts_with("end\t")
        || trimmed.starts_with("end;")
}

/// Returns true if `trimmed` starts with `kw` followed by a non-word character
/// (space, `(`, `!`, or end-of-string), ensuring we don't match prefixes like
/// `do_something` when looking for `do`.
fn starts_with_keyword(trimmed: &str, kw: &str) -> bool {
    if !trimmed.starts_with(kw) {
        return false;
    }
    // If the keyword itself is the whole string, it's a match
    let rest = &trimmed[kw.len()..];
    if rest.is_empty() {
        return true;
    }
    // The character after the keyword must be a non-word character
    let next = rest.as_bytes()[0];
    !next.is_ascii_alphanumeric() && next != b'_'
}

/// Returns true if the trimmed line opens a new block that needs an `end`
/// (def, class, module, do, begin, if, unless, while, until, for, case).
fn opens_block(trimmed: &str) -> bool {
    let keywords = [
        "def", "class", "module", "begin", "do", "case",
        "if", "unless", "while", "until", "for",
    ];
    keywords.iter().any(|kw| starts_with_keyword(trimmed, kw))
}

impl Rule for NegatedIfElseCondition {
    fn name(&self) -> &'static str {
        "Style/NegatedIfElseCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut idx = 0;
        while idx < n {
            let trimmed = lines[idx].trim();
            if is_negated_if(trimmed) && !is_compound_negated_condition(trimmed) {
                // Scan forward to find `else` at the same nesting level (depth=1).
                // Depth starts at 1 because the `if` we found opened a block.
                let if_line = idx;
                let mut depth = 1i32;
                let mut found_else = false;
                let mut j = idx + 1;

                let mut has_elsif = false;
                while j < n {
                    let t = lines[j].trim();

                    if depth == 1 {
                        if is_else(t) {
                            found_else = true;
                            break;
                        }
                        // elsif makes this a multi-branch conditional — not invertible
                        if t.starts_with("elsif") {
                            has_elsif = true;
                            break;
                        }
                    }

                    if is_end(t) {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else if opens_block(t) {
                        depth += 1;
                    }

                    j += 1;
                }

                if found_else && !has_elsif {
                    let line_start = ctx.line_start_offsets[if_line] as u32;
                    let line_end = line_start + lines[if_line].len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: MESSAGE.into(),
                        range: TextRange::new(line_start, line_end),
                        severity: Severity::Warning,
                    });
                }

                // Advance past the block we just scanned to avoid double-flagging
                idx = j + 1;
                continue;
            }

            idx += 1;
        }

        diags
    }
}
