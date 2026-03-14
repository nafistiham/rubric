use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct StringChars;

/// Patterns that should be flagged (all trigger `.chars` suggestion).
const SPLIT_PATTERNS: &[&str] = &[".split(\"\")", ".split('')", ".split(//)"];

impl Rule for StringChars {
    fn name(&self) -> &'static str {
        "Style/StringChars"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (line_idx, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip full-line comments
            if trimmed.starts_with('#') {
                continue;
            }

            // Strip inline comment — find first unquoted `#`
            let code_part = strip_inline_comment(line);
            let line_start = ctx.line_start_offsets[line_idx] as usize;

            for pattern in SPLIT_PATTERNS {
                let mut search_from = 0usize;
                while let Some(pos) = code_part[search_from..].find(pattern) {
                    let abs_pos = search_from + pos;
                    let start = (line_start + abs_pos) as u32;
                    let end = (line_start + abs_pos + pattern.len()) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message:
                            "Use `String#chars` instead of `String#split` with empty string or regex."
                                .into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
                    search_from = abs_pos + pattern.len();
                }
            }
        }

        diags
    }
}

/// Returns the portion of `line` before an unquoted `#` (inline comment).
/// Simple heuristic: track single- and double-quote state, stop at `#`.
fn strip_inline_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut i = 0usize;

    while i < n {
        let b = bytes[i];

        if let Some(delim) = in_string {
            match b {
                b'\\' => i += 1, // skip escaped char
                c if c == delim => in_string = None,
                _ => {}
            }
            i += 1;
            continue;
        }

        match b {
            b'\'' | b'"' => in_string = Some(b),
            b'#' => return &line[..i],
            _ => {}
        }
        i += 1;
    }

    line
}
