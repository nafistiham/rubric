use rubric_core::{LintContext, Rule};
use rubric_rules::style::comparable_clamp::ComparableClamp;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/comparable_clamp/offending.rb");
const PASSING: &str = include_str!("fixtures/style/comparable_clamp/passing.rb");

#[test]
fn detects_array_value_min_max() {
    let src = "lower_bound = [value, min].max\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for '[value, min].max', got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ComparableClamp"));
}

#[test]
fn detects_array_value_max_min() {
    let src = "upper_bound = [value, max].min\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for '[value, max].min', got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ComparableClamp"));
}

#[test]
fn no_violation_for_clamp() {
    let src = "result = value.clamp(min, max)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(diags.is_empty(), "clamp call should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_plain_array_max() {
    let src = "result = [a, b].max\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(diags.is_empty(), "plain [a, b].max should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_plain_array_min() {
    let src = "result = [a, b].min\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(diags.is_empty(), "plain [a, b].min should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_violation_in_offending_fixture() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ComparableClamp"));
}

#[test]
fn no_violation_in_passing_fixture() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = ComparableClamp.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}
