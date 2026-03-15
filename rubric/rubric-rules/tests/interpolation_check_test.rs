use rubric_core::{LintContext, Rule};
use rubric_rules::lint::interpolation_check::InterpolationCheck;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/interpolation_check/offending.rb");
const PASSING: &str = include_str!("fixtures/lint/interpolation_check/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = InterpolationCheck.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/InterpolationCheck"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = InterpolationCheck.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}
