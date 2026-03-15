use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/lint/constant_reassignment.rs"]
mod constant_reassignment;
use constant_reassignment::ConstantReassignment;

const OFFENDING: &str =
    include_str!("fixtures/lint/constant_reassignment/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/constant_reassignment/clean.rb");

#[test]
fn detects_constant_reassignment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violations for reassigned constants, got none"
    );
    assert!(diags
        .iter()
        .all(|d| d.rule == "Lint/ConstantReassignment"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations when constants are only defined once, got: {:?}",
        diags
    );
}

#[test]
fn flags_second_assignment_not_first() {
    let src = "MAX = 100\nMAX = 200\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for MAX reassignment");
    // Should flag only once (the second assignment)
    assert_eq!(diags.len(), 1, "should flag exactly one violation");
    assert!(
        diags[0].message.contains("MAX"),
        "message should name the constant, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_single_assignment() {
    let src = "VERSION = \"1.0.0\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "single assignment should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_equality_comparison() {
    let src = "if MAX == 100\n  puts \"ok\"\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "== comparison should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_comment_line() {
    let src = "MAX = 100\n# MAX = 200 would be bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comment should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn detects_multiple_constants_reassigned() {
    let src = "A = 1\nB = 2\nA = 10\nB = 20\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantReassignment.check_source(&ctx);
    assert_eq!(
        diags.len(),
        2,
        "should flag both A and B reassignments, got: {:?}",
        diags
    );
}

#[test]
fn does_not_flag_lowercase_variables() {
    let src = "max = 100\nmax = 200\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantReassignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "lowercase variables should not be flagged, got: {:?}",
        diags
    );
}
