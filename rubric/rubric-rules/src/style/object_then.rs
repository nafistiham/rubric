use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ObjectThen;

/// Returns true if the byte at `pos` in `bytes` is inside a string literal or comment.
fn is_in_string_or_comment(bytes: &[u8], pos: usize) -> bool {
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
                // Everything after `#` is a comment; pos is inside comment
                return true;
            }
            None => {}
        }
        i += 1;
    }

    in_str.is_some()
}

impl Rule for ObjectThen {
    fn name(&self) -> &'static str {
        "Style/ObjectThen"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let pattern = b".yield_self";

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut search = 0usize;
            while search < bytes.len() {
                if let Some(rel) = bytes[search..]
                    .windows(pattern.len())
                    .position(|w| w == pattern)
                {
                    let abs = search + rel;

                    // Verify word boundary after `.yield_self`
                    // Next char must not be alphanumeric or `_`
                    let after = abs + pattern.len();
                    let after_ok = after >= bytes.len()
                        || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');

                    if after_ok && !is_in_string_or_comment(bytes, abs) {
                        let start = (line_start + abs + 1) as u32; // +1 to skip the `.`
                        let end = (line_start + abs + pattern.len()) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Use then instead of yield_self.".into(),
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

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_yield_self_with_block() {
        let src = "foo.yield_self { |x| x.to_s }\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ObjectThen.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for '.yield_self', got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/ObjectThen"));
    }

    #[test]
    fn detects_yield_self_with_proc() {
        let src = "value.yield_self(&method(:process))\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ObjectThen.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for '.yield_self', got none"
        );
    }

    #[test]
    fn no_violation_for_then() {
        let src = "foo.then { |x| x.to_s }\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ObjectThen.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_yield_self_in_string() {
        let src = "name = \"yield_self_method\"\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ObjectThen.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for yield_self in string, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_yield_self_in_comment() {
        let src = "x = 1 # use .yield_self here\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = ObjectThen.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for yield_self in comment, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending = include_str!("../../tests/fixtures/style/object_then/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = ObjectThen.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/ObjectThen"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing = include_str!("../../tests/fixtures/style/object_then/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = ObjectThen.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}
