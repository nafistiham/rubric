use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

/// Ruby keywords that can legitimately precede `(` with a space — not method calls.
const KEYWORDS: &[&str] = &[
    "if", "unless", "while", "until", "for", "case", "when", "then",
    "do", "rescue", "ensure", "begin", "end", "yield", "return",
    "defined", "not", "and", "or", "in",
];

pub struct SpaceBeforeFirstArg;

impl Rule for SpaceBeforeFirstArg {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeFirstArg"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                // String tracking
                if let Some(delim) = in_string {
                    match b {
                        b'\\' => { j += 2; continue; }
                        c if c == delim => { in_string = None; }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                match b {
                    b'"' | b'\'' | b'`' => { in_string = Some(b); }
                    b'#' => break, // inline comment
                    b' ' => {
                        // Potential `name (` pattern.
                        // Check: next non-space char is `(`.
                        if j + 1 < n && bytes[j + 1] == b'(' {
                            // Look back to find the identifier before the space.
                            if j == 0 {
                                j += 1;
                                continue;
                            }
                            let mut id_end = j - 1;
                            // Skip trailing `!` or `?` (method suffixes)
                            if id_end < n && (bytes[id_end] == b'!' || bytes[id_end] == b'?') {
                                if id_end == 0 { j += 1; continue; }
                                id_end -= 1;
                            }
                            // The char at id_end must be alphanumeric or `_` — otherwise
                            // the space is not preceded by an identifier (e.g. `, (` or `. (`)
                            if !bytes[id_end].is_ascii_alphanumeric() && bytes[id_end] != b'_' {
                                j += 1;
                                continue;
                            }
                            // Collect identifier characters backwards
                            let mut id_start = id_end;
                            while id_start > 0
                                && (bytes[id_start - 1].is_ascii_alphanumeric()
                                    || bytes[id_start - 1] == b'_')
                            {
                                id_start -= 1;
                            }
                            if id_start > id_end {
                                j += 1;
                                continue;
                            }
                            let name_bytes = &bytes[id_start..=id_end];
                            // Must start with a lowercase letter or underscore (method call)
                            if !name_bytes[0].is_ascii_lowercase() && name_bytes[0] != b'_' {
                                j += 1;
                                continue;
                            }
                            let name = std::str::from_utf8(name_bytes).unwrap_or("");
                            // Skip Ruby keywords
                            if KEYWORDS.contains(&name) {
                                j += 1;
                                continue;
                            }
                            // Skip if preceded by `.` (receiver.method (arg) — also a violation
                            // but low-risk), `=` (assignment rhs), or `{` (hash value)
                            // Actually rubocop flags receiver.method (arg) too, so include it.
                            // Skip if preceded by non-identifier chars that indicate keyword context:
                            // e.g. `return (val)`, `raise (err)`
                            // Those are already in KEYWORDS. Safe to proceed.

                            // Skip if this is `def name (` — method definition, not a call
                            let before_id = if id_start >= 4 {
                                let s = std::str::from_utf8(&bytes[..id_start])
                                    .unwrap_or("")
                                    .trim_end();
                                s.ends_with("def") || s.ends_with("def\t")
                            } else {
                                let prefix = std::str::from_utf8(&bytes[..id_start])
                                    .unwrap_or("")
                                    .trim();
                                prefix == "def"
                            };
                            if before_id {
                                j += 1;
                                continue;
                            }

                            let start = (line_start + j) as u32;
                            let end = (line_start + j + 1) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Space found before the first argument of a method call.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::LintContext;

    fn offenses(src: &str) -> Vec<String> {
        let ctx = LintContext::new(std::path::Path::new("test.rb"), src);
        SpaceBeforeFirstArg.check_source(&ctx)
            .into_iter()
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn flags_space_before_paren() {
        assert!(!offenses("foo (bar)").is_empty());
        assert!(!offenses("puts (\"hello\")").is_empty());
    }

    #[test]
    fn allows_no_space() {
        assert!(offenses("foo(bar)").is_empty());
        assert!(offenses("puts(\"hello\")").is_empty());
    }

    #[test]
    fn allows_keywords() {
        assert!(offenses("if (condition)").is_empty());
        assert!(offenses("while (true)").is_empty());
        assert!(offenses("unless (x)").is_empty());
        assert!(offenses("return (value)").is_empty());
    }

    #[test]
    fn allows_method_def() {
        assert!(offenses("def foo (bar)").is_empty());
        assert!(offenses("  def initialize (x, y)").is_empty());
    }

    #[test]
    fn allows_no_args() {
        assert!(offenses("foo").is_empty());
        assert!(offenses("puts").is_empty());
    }
}
