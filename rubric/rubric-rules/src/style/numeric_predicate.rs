use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NumericPredicate;

/// Patterns: (operator + rhs_literal, suggested_predicate)
const PATTERNS: &[(&str, &str)] = &[
    ("== 0", "zero?"),
    ("!= 0", "nonzero?"),
    ("> 0", "positive?"),
    ("< 0", "negative?"),
    (">= 1", "positive?"),
    ("<= -1", "negative?"),
];

fn in_string_or_comment(line: &str, byte_pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < byte_pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return true, // comment
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for NumericPredicate {
    fn name(&self) -> &'static str {
        "Style/NumericPredicate"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;

            for (pattern, predicate) in PATTERNS {
                let mut search_from = 0;
                while let Some(pos) = line[search_from..].find(pattern) {
                    let abs_pos = search_from + pos;

                    if in_string_or_comment(line, abs_pos) {
                        search_from = abs_pos + pattern.len();
                        continue;
                    }

                    // Make sure after the pattern there's no digit (avoid `== 00` etc.)
                    let after = abs_pos + pattern.len();
                    if after < line.len() {
                        let next_byte = line.as_bytes()[after];
                        if next_byte.is_ascii_digit() || next_byte == b'_' {
                            search_from = abs_pos + 1;
                            continue;
                        }
                    }

                    // Extract LHS — go backwards from `abs_pos` skipping whitespace
                    let before = line[..abs_pos].trim_end();
                    if before.is_empty() {
                        search_from = abs_pos + pattern.len();
                        continue;
                    }

                    // Grab a reasonable LHS token (last word/expression segment)
                    let lhs_end = before.len();
                    let lhs_start = before
                        .rfind(|c: char| {
                            !c.is_alphanumeric() && c != '_' && c != '.' && c != '?' && c != '!'
                        })
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let lhs = &before[lhs_start..lhs_end];

                    if lhs.is_empty() {
                        search_from = abs_pos + pattern.len();
                        continue;
                    }

                    let flag_start = (line_start + abs_pos) as u32;
                    let flag_end = (line_start + abs_pos + pattern.len()) as u32;

                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: format!(
                            "Use `{}.{}` instead of `{} {}`.",
                            lhs, predicate, lhs, pattern
                        ),
                        range: TextRange::new(flag_start, flag_end),
                        severity: Severity::Warning,
                    });

                    search_from = abs_pos + pattern.len();
                }
            }
        }

        diags
    }
}
