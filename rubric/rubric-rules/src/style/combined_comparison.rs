use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CombinedComparison;

const MESSAGE: &str = "Use the spaceship operator `<=>` instead of an explicit comparison.";

/// Returns the byte position of the first unquoted `#` comment character,
/// or `line.len()` if there is no comment.
fn code_end(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
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
            None if bytes[i] == b'#' => return i,
            None => {}
        }
        i += 1;
    }
    line.len()
}

/// Returns `true` if the code portion of `line` matches the heuristic for a
/// combined-comparison pattern, i.e. `a > b ? 1 : a < b ? -1 : 0` or the
/// parenthesised variant `a > b ? 1 : (a < b ? -1 : 0)`.
///
/// Heuristic: the code portion must contain:
/// - both `> ` and `< ` (greater-than and less-than with a space, to avoid `<=>`)
/// - both `?` and `:` (ternary)
/// - the numeric literals `1`, `-1`, and `0` in some form
fn is_combined_comparison(code: &str) -> bool {
    // Must have ternary operators
    if !code.contains('?') || !code.contains(':') {
        return false;
    }

    // Must have explicit `>` and `<` comparisons (not `<=>`)
    // We check for `> ` and `< ` to avoid matching `<=>` as both `<` and `>`
    let has_gt = code.contains("> ");
    let has_lt = code.contains("< ");
    if !has_gt || !has_lt {
        return false;
    }

    // Must contain all three magic numbers: 1, -1, 0
    // Check for ` 1` or `? 1` or `(1` and similar
    let has_pos1 = code.contains(" 1") || code.contains("? 1") || code.contains("?1");
    let has_neg1 = code.contains("-1");
    let has_zero = code.contains(" 0") || code.contains(": 0") || code.contains(":0");

    has_pos1 && has_neg1 && has_zero
}

impl Rule for CombinedComparison {
    fn name(&self) -> &'static str {
        "Style/CombinedComparison"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let end = code_end(line);
            let code = &line[..end];

            if is_combined_comparison(code) {
                let line_start = ctx.line_start_offsets[i] as usize;
                // Flag from the start of the line's code content to the end of code
                let start = (line_start + (line.len() - line.trim_start().len())) as u32;
                let stop = (line_start + end) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: MESSAGE.into(),
                    range: TextRange::new(start, stop),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
