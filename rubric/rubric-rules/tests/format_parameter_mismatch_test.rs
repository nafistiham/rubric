use rubric_core::{LintContext, Rule};
use rubric_rules::lint::format_parameter_mismatch::FormatParameterMismatch;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/format_parameter_mismatch/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FormatParameterMismatch.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/FormatParameterMismatch"));
}

#[test]
fn no_violation_on_clean() {
    let src = "sprintf(\"%s %s\", name, age)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FormatParameterMismatch.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
