use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct TrivialAccessors;

/// Check if a line is a comment (after trimming leading whitespace).
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Extract the method name from a `def` line.
/// Returns `Some((name, is_setter, param))` where:
/// - `name` is the method name without `=` suffix for setters
/// - `is_setter` is true when the method ends with `=`
/// - `param` is the setter parameter name (e.g. "val") or empty for getters
fn parse_def_line(line: &str) -> Option<(String, bool, String)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("def ")?;

    // Skip `self.` prefix
    let rest = rest.strip_prefix("self.").unwrap_or(rest);

    // Find where the name ends: `(`, `;`, ` `, `\t`, or EOL
    let name_end = rest
        .find(|c: char| c == '(' || c == ';' || c == ' ' || c == '\t')
        .unwrap_or(rest.len());
    let raw_name = &rest[..name_end];

    if raw_name.is_empty() {
        return None;
    }

    // Setter: name ends with `=`
    if let Some(setter_name) = raw_name.strip_suffix('=') {
        if setter_name.is_empty() {
            return None;
        }
        // Extract parameter name from `(param)` or just `param`
        let after_name = &rest[name_end..];
        let param = extract_setter_param(after_name);
        return Some((setter_name.to_string(), true, param));
    }

    // Getter: must have no params (no `(` right after name, or empty parens)
    let after_name = rest[name_end..].trim_start();
    if after_name.starts_with('(') {
        // Allow `()` but not `(param)`
        let after_paren = after_name[1..].trim_start();
        if !after_paren.starts_with(')') {
            return None; // has parameters
        }
    }

    Some((raw_name.to_string(), false, String::new()))
}

/// Extract the first identifier from `(param)` or ` param` style.
fn extract_setter_param(s: &str) -> String {
    let s = s.trim_start();
    let inner = if s.starts_with('(') {
        let close = s.find(')').unwrap_or(s.len());
        s[1..close].trim()
    } else {
        s.split(|c: char| c == ';' || c == ' ' || c == '\t')
            .next()
            .unwrap_or("")
            .trim()
    };
    // Take just the first word
    inner
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Check if a body line is exactly `@name` (trivial reader body).
fn is_trivial_reader_body(body: &str, name: &str) -> bool {
    let t = body.trim();
    t == format!("@{}", name)
}

/// Check if a body line is exactly `@name = param` (trivial writer body).
fn is_trivial_writer_body(body: &str, name: &str, param: &str) -> bool {
    let t = body.trim();
    // Accept `@name = param` with any spacing around `=`
    let expected = format!("@{}", name);
    if !t.starts_with(expected.as_str()) {
        return false;
    }
    let after = t[expected.len()..].trim_start();
    if !after.starts_with('=') {
        return false;
    }
    // Make sure it's `=` and not `==`
    let after_eq = after[1..].trim_start();
    if after.starts_with("==") {
        return false;
    }
    // The right-hand side must equal param (if param is known)
    if param.is_empty() {
        return false;
    }
    after_eq == param
}

/// Check a one-liner `def foo; @foo; end` or `def foo=(v); @foo = v; end`.
fn check_one_liner(line: &str) -> Option<(&'static str, usize)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("def ") {
        return None;
    }

    // Getter one-liner: `def <name>; @<name>; end`
    // Split by `;` to get segments
    let parts: Vec<&str> = trimmed.splitn(4, ';').collect();
    if parts.len() < 3 {
        return None;
    }

    let def_part = parts[0].trim();
    let body_part = parts[1].trim();
    let end_part = parts[2].trim();

    if end_part != "end" && !end_part.starts_with("end ") && !end_part.starts_with("end\t") {
        return None;
    }

    // Try to parse the def segment as a method definition
    if let Some((name, is_setter, param)) = parse_def_line(def_part) {
        let def_col = line.len() - trimmed.len();

        if is_setter {
            if is_trivial_writer_body(body_part, &name, &param) {
                return Some(("Use `attr_writer` to define trivial writer methods.", def_col));
            }
        } else if is_trivial_reader_body(body_part, &name) {
            return Some(("Use `attr_reader` to define trivial reader methods.", def_col));
        }
    }

    None
}

impl Rule for TrivialAccessors {
    fn name(&self) -> &'static str {
        "Style/TrivialAccessors"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // State machine for multi-line detection:
        // None = not inside a def
        // Some((name, is_setter, param, def_line_idx)) = inside a def header
        enum State {
            Outside,
            // After seeing the header, waiting for the body line
            WaitingBody {
                name: String,
                is_setter: bool,
                param: String,
                def_line_idx: usize,
            },
            // After seeing a plausible body, waiting for `end`
            WaitingEnd {
                def_line_idx: usize,
                message: &'static str,
            },
        }

        let mut state = State::Outside;

        for (i, line) in ctx.lines.iter().enumerate() {
            if is_comment_line(line) {
                // Reset state on unexpected content
                match state {
                    State::Outside => {}
                    _ => {
                        state = State::Outside;
                    }
                }
                continue;
            }

            let trimmed = line.trim();

            // Check one-liners first (regardless of current state)
            if trimmed.starts_with("def ") && trimmed.contains(';') {
                if let Some((msg, col)) = check_one_liner(line) {
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let start = (line_start + col) as u32;
                    let end = start + 3; // `def`
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: msg.to_string(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    state = State::Outside;
                    continue;
                }
            }

            match state {
                State::Outside => {
                    if trimmed.starts_with("def ") && !trimmed.contains(';') {
                        if let Some((name, is_setter, param)) = parse_def_line(line) {
                            // Skip methods starting with `_`
                            if !name.starts_with('_') {
                                state = State::WaitingBody {
                                    name,
                                    is_setter,
                                    param,
                                    def_line_idx: i,
                                };
                                continue;
                            }
                        }
                    }
                }
                State::WaitingBody {
                    ref name,
                    is_setter,
                    ref param,
                    def_line_idx,
                } => {
                    if trimmed.is_empty() {
                        // Blank lines reset state — def body should be immediate
                        state = State::Outside;
                        continue;
                    }

                    let is_trivial = if is_setter {
                        is_trivial_writer_body(trimmed, name, param)
                    } else {
                        is_trivial_reader_body(trimmed, name)
                    };

                    if is_trivial {
                        let msg: &'static str = if is_setter {
                            "Use `attr_writer` to define trivial writer methods."
                        } else {
                            "Use `attr_reader` to define trivial reader methods."
                        };
                        state = State::WaitingEnd {
                            def_line_idx,
                            message: msg,
                        };
                    } else {
                        // Not trivial — this def is complex, reset
                        state = State::Outside;
                    }
                }
                State::WaitingEnd {
                    def_line_idx,
                    message,
                } => {
                    if trimmed == "end" {
                        let line_start = ctx.line_start_offsets[def_line_idx] as usize;
                        let def_line = &ctx.lines[def_line_idx];
                        let def_trimmed = def_line.trim_start();
                        let col = def_line.len() - def_trimmed.len();
                        let start = (line_start + col) as u32;
                        let end = start + 3; // `def`
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: message.to_string(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                        state = State::Outside;
                    } else {
                        // Something else before `end` — not trivial
                        state = State::Outside;
                    }
                }
            }
        }

        diags
    }
}
