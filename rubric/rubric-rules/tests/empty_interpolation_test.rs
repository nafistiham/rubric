use rubric_core::{LintContext, Rule};
use rubric_rules::lint::empty_interpolation::EmptyInterpolation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/empty_interpolation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/EmptyInterpolation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = \"hello #{name} world\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
