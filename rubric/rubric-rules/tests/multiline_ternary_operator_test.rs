use rubric_core::{LintContext, Rule};
use rubric_rules::style::multiline_ternary_operator::MultilineTernaryOperator;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/multiline_ternary_operator/offending.rb");
const PASSING: &str = include_str!("fixtures/style/multiline_ternary_operator/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineTernaryOperator.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb, got none");
    assert!(
        diags.iter().all(|d| d.rule == "Style/MultilineTernaryOperator"),
        "unexpected rule name"
    );
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = MultilineTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on passing.rb, got: {:?}", diags);
}

#[test]
fn flags_multiline_ternary_with_space_before_question() {
    // Classic multiline ternary: condition ends with ` ?`
    let src = "result = some_condition ?\n  value_if_true :\n  value_if_false\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineTernaryOperator.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Style/MultilineTernaryOperator");
}

#[test]
fn does_not_flag_predicate_method() {
    // foo? is a predicate method, not a ternary
    let src = "if foo?\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "predicate method should not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_single_line_ternary() {
    // Single-line ternary is fine for this cop
    let src = "result = condition ? value_a : value_b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineTernaryOperator.check_source(&ctx);
    assert!(diags.is_empty(), "single-line ternary must not be flagged: {:?}", diags);
}

#[test]
fn message_contains_if_or_unless() {
    let src = "result = some_condition ?\n  value_if_true :\n  value_if_false\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineTernaryOperator.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("if") || diags[0].message.contains("unless"),
        "message should mention `if` or `unless`: {}",
        diags[0].message
    );
}
