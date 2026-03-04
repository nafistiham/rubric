use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UselessSetterCall;

/// Returns true if `s` looks like a bare local variable name (all lowercase,
/// may contain underscores/digits, no `(`, `.`, `[`, `?`, `!`, spaces, or `:`).
fn is_bare_local(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // Must start with lowercase letter or underscore (not a constant, not `@`, `$`, `:`)
    let first = s.chars().next().unwrap();
    if !first.is_ascii_lowercase() && first != '_' {
        return false;
    }
    // Must consist only of word characters — no calls, indexing, spaces, operators
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

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

            // Only enter regular method definitions — skip setter methods (`def foo=`)
            // because they MUST write to `self` as their purpose.
            if trimmed.starts_with("def ") && !trimmed.contains("def self.") {
                let method_name = trimmed.trim_start_matches("def ").trim();
                // Skip setter method definitions (ending in `=` before optional params)
                // e.g. `def foo=(value)` or `def foo=`
                if method_name.contains('=') {
                    i += 1;
                    continue;
                }

                let def_line = i;
                let mut depth = 1usize;
                let mut j = i + 1;

                while j < n && depth > 0 {
                    let t = lines[j].trim();

                    // Things that open a new depth level
                    if t.starts_with("def ")
                        || t.starts_with("if ")
                        || t.starts_with("unless ")
                        || t.starts_with("while ")
                        || t.starts_with("until ")
                        || t.starts_with("for ")
                        || t.starts_with("case ")
                        || t.starts_with("class ")
                        || t.starts_with("module ")
                        || t == "if"
                        || t == "unless"
                        || t == "begin"
                        || t == "do"
                        || t.ends_with(" do")
                        || t.ends_with(" do |")
                        || (t.contains(" do |") && !t.starts_with('#'))
                        || t.starts_with("begin")
                    {
                        depth += 1;
                    }

                    // `end` on its own closes one level
                    if t == "end" || t.starts_with("end ") || t.starts_with("end#") {
                        depth -= 1;
                    }

                    j += 1;
                }

                let def_end = j; // line index AFTER the closing `end`

                // Find the last non-empty, non-comment line before the closing `end`
                // (which is at def_end - 1).
                let closing_end_idx = def_end.saturating_sub(1);
                let mut last_content = closing_end_idx.saturating_sub(1);
                while last_content > def_line {
                    let t = lines[last_content].trim();
                    if !t.is_empty() && !t.starts_with('#') {
                        break;
                    }
                    if last_content == 0 {
                        break;
                    }
                    last_content -= 1;
                }

                if last_content > def_line {
                    let last_line = lines[last_content].trim();

                    // Must start with `self.something =`
                    if last_line.starts_with("self.") && last_line.contains(" = ") {
                        // Extract the RHS: everything after the first ` = `
                        if let Some(eq_pos) = last_line.find(" = ") {
                            let rhs = last_line[eq_pos + 3..].trim();

                            // Only flag when:
                            // 1. No modifier conditional on the setter line (`if`/`unless` at end)
                            // 2. RHS is a bare local variable (not a method call / complex expr)
                            let has_modifier = rhs.contains(" if ")
                                || rhs.contains(" unless ")
                                || last_line.contains(" if ")
                                || last_line.contains(" unless ");

                            // A method call has `(`, `.`, `[`, `?`, `!` in RHS,
                            // or starts with `@`, `$`, `:`, `"`, `'`, a digit, or uppercase.
                            // A bare local var is only word chars starting with lowercase/underscore.
                            let rhs_is_bare_local = is_bare_local(rhs);

                            if !has_modifier && rhs_is_bare_local {
                                let indent = lines[last_content].len()
                                    - lines[last_content].trim_start().len();
                                let line_start =
                                    ctx.line_start_offsets[last_content] as usize;
                                let pos = (line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message:
                                        "Useless setter call at end of method; return value is discarded."
                                            .into(),
                                    range: TextRange::new(
                                        pos,
                                        pos + last_line.len() as u32,
                                    ),
                                    severity: Severity::Warning,
                                });
                            }
                        }
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
