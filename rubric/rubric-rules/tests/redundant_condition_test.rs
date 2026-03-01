use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_condition::RedundantCondition;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/redundant_condition/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantCondition"));
}

#[test]
fn no_violation_on_clean() {
    let src = "result = condition\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
