use rubric_core::{LintContext, Rule};
use rubric_rules::style::negated_if_else_condition::NegatedIfElseCondition;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/negated_if_else_condition/offending.rb");
const PASSING: &str = include_str!("fixtures/style/negated_if_else_condition/passing.rb");

#[test]
fn detects_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NegatedIfElseCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedIfElseCondition"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = NegatedIfElseCondition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}

#[test]
fn flags_if_bang_with_else() {
    let src = "if !condition\n  a\nelse\n  b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIfElseCondition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for if ! with else");
}

#[test]
fn does_not_flag_if_bang_without_else() {
    let src = "if !condition\n  a\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIfElseCondition.check_source(&ctx);
    assert!(diags.is_empty(), "if ! without else should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_positive_if_with_else() {
    let src = "if condition\n  a\nelse\n  b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIfElseCondition.check_source(&ctx);
    assert!(diags.is_empty(), "positive if with else should not be flagged, got: {:?}", diags);
}

#[test]
fn message_mentions_invert() {
    let src = "if !condition\n  a\nelse\n  b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIfElseCondition.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.to_lowercase().contains("invert") || diags[0].message.contains("negat"),
        "message should mention invert or negation, got: {}",
        diags[0].message
    );
}
