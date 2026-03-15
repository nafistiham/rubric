use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ArrayJoin;

/// Returns true if the byte at `pos` inside `bytes` is within a string literal
/// (single- or double-quoted) or after a `#` comment marker.
fn in_string_or_comment_at(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos && i < bytes.len() {
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
            None if bytes[i] == b'#' => {
                // Real comment — position is in a comment, treat as not code
                return true;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

impl Rule for ArrayJoin {
    fn name(&self) -> &'static str {
        "Style/ArrayJoin"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Patterns for .join("") and .join('')
        const PATTERNS: &[&[u8]] = &[b".join(\"\")", b".join('')"];

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            for pattern in PATTERNS {
                let mut search = 0usize;
                while search < bytes.len() {
                    if let Some(rel) = bytes[search..]
                        .windows(pattern.len())
                        .position(|w| w == *pattern)
                    {
                        let abs = search + rel;

                        if !in_string_or_comment_at(bytes, abs) {
                            // Flag the `.join(...)` portion
                            let start = (line_start + abs) as u32;
                            let end = start + pattern.len() as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message:
                                    "Use Array#join without arguments instead of Array#join with empty string."
                                        .into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                        }
                        search = abs + pattern.len();
                    } else {
                        break;
                    }
                }
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_join_with_empty_double_quotes() {
        let src = "arr.join(\"\")\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for .join(\"\"), got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/ArrayJoin"));
    }

    #[test]
    fn detects_join_with_empty_single_quotes() {
        let src = "arr.join('')\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for .join(''), got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/ArrayJoin"));
    }

    #[test]
    fn no_violation_for_join_with_separator() {
        let src = "arr.join(\", \")\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_join_no_args() {
        let src = "arr.join\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_join_in_string() {
        let src = "x = \".join('')\"\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations for join in string, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_join_in_comment() {
        let src = "x = 1 # arr.join('')\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations for join in comment, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending = include_str!("../../tests/fixtures/style/array_join/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/ArrayJoin"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing = include_str!("../../tests/fixtures/style/array_join/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = ArrayJoin.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}
