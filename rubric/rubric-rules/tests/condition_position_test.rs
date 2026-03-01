use rubric_core::{LintContext, Rule};
use rubric_rules::layout::condition_position::ConditionPosition;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/condition_position/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ConditionPosition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/ConditionPosition"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if foo\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConditionPosition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
