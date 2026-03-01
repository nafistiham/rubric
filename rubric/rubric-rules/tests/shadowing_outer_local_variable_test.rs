use rubric_core::{LintContext, Rule};
use rubric_rules::lint::shadowing_outer_local_variable::ShadowingOuterLocalVariable;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/shadowing_outer_local_variable/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/ShadowingOuterLocalVariable"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1\n[1, 2].each { |y| puts y }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ShadowingOuterLocalVariable.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
