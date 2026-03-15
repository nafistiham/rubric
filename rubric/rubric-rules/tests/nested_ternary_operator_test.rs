use rubric_core::{LintContext, Rule};
use rubric_rules::style::nested_ternary_operator::NestedTernaryOperator;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/nested_ternary_operator/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/nested_ternary_operator/passing.rb");

#[test]
fn detects_nested_ternary() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for nested ternary, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/NestedTernaryOperator"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_suffix_nested_ternary() {
    let src = "a ? b : c ? d : e\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(!diags.is_empty(), "a ? b : c ? d : e should be flagged");
}

#[test]
fn detects_prefix_nested_ternary() {
    let src = "a ? b ? c : d : e\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(!diags.is_empty(), "a ? b ? c : d : e should be flagged");
}

#[test]
fn no_violation_for_single_ternary() {
    let src = "x = a ? b : c\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "single ternary should not be flagged");
}

#[test]
fn no_violation_for_comment() {
    let src = "# a ? b : c ? d : e\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged");
}

#[test]
fn no_violation_for_hash_rockets() {
    let src = "map = { :foo => 1, :bar => 2 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "hash rockets should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_predicate_method_single_ternary() {
    // foo? is a predicate but only one ternary
    let src = "x = foo? ? bar : baz\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "single ternary with predicate method should not be flagged");
}

#[test]
fn detects_three_ternary_questions() {
    // All three offending lines should be caught
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NestedTernaryOperator.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations, got: {:?}", diags);
}
