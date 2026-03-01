use rubric_core::{LintContext, Rule};
use rubric_rules::style::conditional_assignment::ConditionalAssignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/conditional_assignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ConditionalAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ConditionalAssignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = foo ? 1 : 2\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConditionalAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
