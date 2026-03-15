use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_assignment::RedundantAssignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/redundant_assignment/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/redundant_assignment/corrected.rb");

#[test]
fn detects_redundant_assignment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantAssignment"));
}

#[test]
fn detects_both_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantAssignment.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations, got: {:?}", diags);
}

#[test]
fn no_violation_on_corrected() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on corrected code, got: {:?}", diags);
}

#[test]
fn no_violation_when_var_used_elsewhere() {
    // result is used in two places — not redundant
    let src = "def foo\n  result = something\n  puts result\n  result\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation when var is used elsewhere, got: {:?}", diags);
}

#[test]
fn no_violation_for_instance_variables() {
    // @result = ... followed by @result — not a local var, skip
    let src = "def foo\n  @result = something\n  @result\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantAssignment.check_source(&ctx);
    assert!(diags.is_empty(), "instance var should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_inline_assignment_followed_by_var() {
    let src = "def compute\n  total = a + b + c\n  total\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantAssignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for assign-then-return pattern");
    assert_eq!(diags[0].rule, "Style/RedundantAssignment");
}
