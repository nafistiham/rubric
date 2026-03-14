use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct Semicolon;

/// Scan through a line byte-by-byte tracking string and comment state.
/// Returns the position of the first `;` found outside a string or comment,
/// or `None` if no such semicolon exists.
fn first_semicolon_outside_string_comment(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;

    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                // Skip escaped character
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
                // Real comment — nothing after this is code
                return None;
            }
            None if bytes[i] == b';' => {
                return Some(i);
            }
            None => {}
        }
        i += 1;
    }
    None
}

/// Returns true if the `;` at `pos` is a trailing semicolon (only whitespace after it).
fn is_trailing_semicolon(line: &str, pos: usize) -> bool {
    line[pos + 1..].trim().is_empty()
}

impl Rule for Semicolon {
    fn name(&self) -> &'static str {
        "Style/Semicolon"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;

            // Find the first `;` outside strings/comments
            if let Some(pos) = first_semicolon_outside_string_comment(line) {
                // Skip trailing semicolons (nothing substantive after them)
                if is_trailing_semicolon(line, pos) {
                    continue;
                }

                // Skip if nothing before the semicolon (just whitespace)
                if line[..pos].trim().is_empty() {
                    continue;
                }

                let start = (line_start + pos) as u32;
                let end = start + 1;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Do not use semicolons to terminate expressions.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
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
    fn detects_multiple_statements_on_one_line() {
        let src = "x = 1; y = 2\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'x = 1; y = 2', got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/Semicolon"));
    }

    #[test]
    fn detects_multiple_semicolons() {
        let src = "a = 1; b = 2; c = 3\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations for multiple semicolons"
        );
    }

    #[test]
    fn no_violation_for_clean_code() {
        let src = "x = 1\ny = 2\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_semicolon_in_string() {
        let src = "greeting = \"hello; world\"\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for semicolon in string, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_semicolon_in_comment() {
        let src = "x = 1 # this; is a comment\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for semicolon in comment, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_trailing_semicolon() {
        let src = "x = 1;\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for trailing semicolon, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending = include_str!("../../tests/fixtures/style/semicolon/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/Semicolon"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing = include_str!("../../tests/fixtures/style/semicolon/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = Semicolon.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}
