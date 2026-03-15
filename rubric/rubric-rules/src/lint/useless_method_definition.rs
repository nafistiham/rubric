use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UselessMethodDefinition;

/// Returns true if `line` is a bare `super` or empty-arg `super()` call —
/// the only cases RuboCop considers "useless" (pure pass-through with no modification).
/// `super(args)` and `super args` are NOT flagged: the args could differ from the
/// method signature, making the definition meaningful.
fn is_super_call(line: &str) -> bool {
    let t = line.trim();
    if !t.starts_with("super") {
        return false;
    }
    let after = &t["super".len()..];
    // Bare `super` (no args — Ruby passes all params transparently)
    if after.is_empty() {
        return true;
    }
    // `super()` — explicit empty args, still a pure pass-through
    if after == "()" {
        return true;
    }
    false
}

impl Rule for UselessMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/UselessMethodDefinition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim_start();

            // Match a `def` line that starts a method (not a one-liner with `end` on same line)
            if trimmed.starts_with("def ") || trimmed == "def" {
                // Skip one-liner defs that have `end` on the same line
                if trimmed.contains(';') && trimmed.contains("end") {
                    i += 1;
                    continue;
                }

                // Find the next non-blank line
                let mut body_idx = i + 1;
                while body_idx < n && lines[body_idx].trim().is_empty() {
                    body_idx += 1;
                }

                if body_idx >= n {
                    i += 1;
                    continue;
                }

                let body_line = lines[body_idx].trim();

                if !is_super_call(body_line) {
                    i += 1;
                    continue;
                }

                // Find the next non-blank line after the super call
                let mut end_idx = body_idx + 1;
                while end_idx < n && lines[end_idx].trim().is_empty() {
                    end_idx += 1;
                }

                if end_idx >= n {
                    i += 1;
                    continue;
                }

                let end_line = lines[end_idx].trim();

                if end_line == "end" {
                    // Flag the `def` line
                    let line_start = ctx.line_start_offsets[i] as u32;
                    let line_end = line_start + lines[i].len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Useless method definition detected.".into(),
                        range: TextRange::new(line_start, line_end),
                        severity: Severity::Warning,
                    });
                    // Skip past end
                    i = end_idx + 1;
                    continue;
                }
            }

            i += 1;
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::LintContext;
    use std::path::Path;

    #[test]
    fn detects_bare_super_only_body() {
        let src = "def foo\n  super\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = UselessMethodDefinition.check_source(&ctx);
        assert!(!diags.is_empty(), "expected violation for bare super body");
        assert!(diags.iter().all(|d| d.rule == "Lint/UselessMethodDefinition"));
    }

    #[test]
    fn detects_super_in_method_with_args() {
        let src = "def bar(x, y)\n  super\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = UselessMethodDefinition.check_source(&ctx);
        assert!(!diags.is_empty(), "expected violation for super-only body with args");
    }

    #[test]
    fn no_violation_when_body_has_extra_statements() {
        let src = "def foo\n  super\n  do_extra_work\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = UselessMethodDefinition.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
    }

    #[test]
    fn no_violation_when_super_result_is_used() {
        let src = "def bar\n  modified = transform(super)\n  modified\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = UselessMethodDefinition.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_normal_method() {
        let src = "def foo\n  bar\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = UselessMethodDefinition.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
    }

    #[test]
    fn detects_multiple_useless_methods() {
        let src = "def foo\n  super\nend\n\ndef bar(x, y)\n  super\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = UselessMethodDefinition.check_source(&ctx);
        assert_eq!(diags.len(), 2, "expected 2 violations; got: {:?}", diags);
    }
}
