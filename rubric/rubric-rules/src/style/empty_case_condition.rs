use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct EmptyCaseCondition;

impl Rule for EmptyCaseCondition {
    fn name(&self) -> &'static str {
        "Style/EmptyCaseCondition"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let offsets = &ctx.line_start_offsets;

        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim();

            // Skip comment lines
            if trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Match a line that is exactly `case` with no value after it
            if trimmed == "case" {
                // Find the next non-blank line and check if it starts with `when`
                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j < lines.len() && lines[j].trim().starts_with("when") {
                    let line_start = offsets[i] as usize;
                    let indent = lines[i].len() - lines[i].trim_start().len();
                    let start = (line_start + indent) as u32;
                    let end = start + "case".len() as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Do not use empty case condition, instead use an if expression."
                            .into(),
                        range: TextRange::new(start, end),
                        severity: Severity::Warning,
                    });
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
    use std::path::Path;

    #[test]
    fn detects_empty_case_followed_by_when() {
        let src = "case\nwhen x > 5\n  puts 'big'\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = EmptyCaseCondition.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for empty case, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/EmptyCaseCondition"));
    }

    #[test]
    fn no_violation_for_case_with_value() {
        let src = "case x\nwhen 1\n  puts 'one'\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = EmptyCaseCondition.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_case_without_when() {
        // `case` alone with no `when` following — not our concern
        let src = "case\n  foo\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = EmptyCaseCondition.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_in_comment() {
        let src = "# case\n# when foo\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = EmptyCaseCondition.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations for comment line, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending =
            include_str!("../../tests/fixtures/style/empty_case_condition/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = EmptyCaseCondition.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/EmptyCaseCondition"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing =
            include_str!("../../tests/fixtures/style/empty_case_condition/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = EmptyCaseCondition.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}
