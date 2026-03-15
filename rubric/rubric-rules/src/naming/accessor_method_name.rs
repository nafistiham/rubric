use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct AccessorMethodName;

/// Count the number of arguments in the parameter list portion of a def line.
/// `remainder` is the part after the method name, e.g. `(a, b)`, `()`, ` a`, or empty.
/// Returns 0 for no args, 1 for one arg, etc.
fn count_method_args(remainder: &str) -> usize {
    let trimmed = remainder.trim_start();
    if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
        // def foo  or  def foo; end  — zero arguments
        return 0;
    }
    if trimmed.starts_with("()") {
        // def foo() — zero arguments
        return 0;
    }
    // Collect the argument list: either `(...)` or bare `a, b` up to end
    let inside = if let Some(rest) = trimmed.strip_prefix('(') {
        // Find matching closing paren
        if let Some(end) = rest.find(')') { &rest[..end] } else { rest }
    } else {
        // No parens: `def foo a, b` — take up to comment or end
        let end = trimmed.find('#').unwrap_or(trimmed.len());
        trimmed[..end].trim_end()
    };
    let inside = inside.trim();
    if inside.is_empty() {
        return 0;
    }
    // Count top-level commas (depth 0) + 1
    let bytes = inside.as_bytes();
    let mut depth = 0i32;
    let mut count = 1usize;
    for &b in bytes {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => { if depth > 0 { depth -= 1; } }
            b',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

impl Rule for AccessorMethodName {
    fn name(&self) -> &'static str {
        "Naming/AccessorMethodName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with('#') {
                continue;
            }

            // Must start with `def `
            let after_def = if let Some(rest) = trimmed.strip_prefix("def ") {
                rest
            } else if let Some(rest) = trimmed.strip_prefix("def\t") {
                rest
            } else {
                continue;
            };

            // Skip self. prefix
            let method_part = after_def.strip_prefix("self.").unwrap_or(after_def);

            // Extract method name up to (, space, tab, ;, or end
            let name_end = method_part
                .find(|c: char| c == '(' || c == ' ' || c == '\t' || c == '\n' || c == ';')
                .unwrap_or(method_part.len());
            let method_name = &method_part[..name_end];

            if method_name.is_empty() {
                continue;
            }

            // Count arguments from the remainder after the method name
            let remainder = &method_part[name_end..];
            let arg_count = count_method_args(remainder);

            let (flagged, msg) = if method_name.starts_with("get_") && method_name.len() > 4 && arg_count == 0 {
                (true, "Do not prefix reader method names with `get_`.".to_string())
            } else if method_name.starts_with("set_") && method_name.len() > 4 && arg_count == 1 {
                (true, "Do not prefix writer method names with `set_`.".to_string())
            } else {
                (false, String::new())
            };

            if flagged {
                let line_start = ctx.line_start_offsets[i] as usize;
                let def_col = line.len() - trimmed.len();
                let start = (line_start + def_col) as u32;
                let end = start + 3; // `def`
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: msg,
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
