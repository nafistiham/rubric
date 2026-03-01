use rubric_core::{LintContext, Rule};
use rubric_rules::lint::self_assignment::SelfAssignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/self_assignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SelfAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/SelfAssignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1\ny = x\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SelfAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
