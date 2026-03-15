use rubric_core::{LintContext, Rule};
use rubric_rules::lint::rescue_type::RescueType;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/rescue_type/offending.rb");
const PASSING: &str = include_str!("fixtures/lint/rescue_type/passing.rb");

#[test]
fn detects_rescue_nil() {
    let src = "begin\n  foo\nrescue nil\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueType.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for 'rescue nil', got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/RescueType"));
}

#[test]
fn detects_rescue_true() {
    let src = "begin\n  foo\nrescue true\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueType.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for 'rescue true', got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/RescueType"));
}

#[test]
fn detects_rescue_false() {
    let src = "begin\n  foo\nrescue false\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueType.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for 'rescue false', got none");
}

#[test]
fn detects_rescue_integer_literal() {
    let src = "begin\n  foo\nrescue 42\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueType.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for 'rescue 42', got none");
}

#[test]
fn no_violation_for_standard_error() {
    let src = "begin\n  foo\nrescue StandardError => e\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueType.check_source(&ctx);
    assert!(diags.is_empty(), "StandardError rescue should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_multiple_exception_classes() {
    let src = "begin\n  foo\nrescue ArgumentError, TypeError => e\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueType.check_source(&ctx);
    assert!(diags.is_empty(), "ArgumentError, TypeError should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_violation_in_offending_fixture() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RescueType.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/RescueType"));
}

#[test]
fn no_violation_in_passing_fixture() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = RescueType.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}
