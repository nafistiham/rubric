use rubric_core::{LintContext, Rule};
use rubric_rules::lint::mixed_case_range::MixedCaseRange;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/mixed_case_range/offending.rb");
const PASSING: &str = include_str!("fixtures/lint/mixed_case_range/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MixedCaseRange.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb, got none");
    assert!(
        diags.iter().all(|d| d.rule == "Lint/MixedCaseRange"),
        "unexpected rule name"
    );
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = MixedCaseRange.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on passing.rb, got: {:?}", diags);
}

#[test]
fn flags_uppercase_to_lowercase_range() {
    // A-z is the classic problematic range
    let src = "str.match?(/[A-z]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation for A-z, got: {:?}", diags);
    assert_eq!(diags[0].rule, "Lint/MixedCaseRange");
}

#[test]
fn flags_lowercase_to_uppercase_range() {
    // a-Z is also problematic (invalid but still mixed case)
    let src = "str.match?(/[a-Z]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation for a-Z, got: {:?}", diags);
}

#[test]
fn does_not_flag_all_uppercase_range() {
    let src = "str.match?(/[A-Z]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert!(diags.is_empty(), "A-Z should not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_all_lowercase_range() {
    let src = "str.match?(/[a-z]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert!(diags.is_empty(), "a-z should not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_digit_range() {
    let src = "str.match?(/[0-9]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert!(diags.is_empty(), "0-9 should not be flagged: {:?}", diags);
}

#[test]
fn message_mentions_upper_and_lower_case() {
    let src = "str.match?(/[A-z]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("upper") || diags[0].message.contains("lower") || diags[0].message.contains("mixes"),
        "message should mention case mixing: {}",
        diags[0].message
    );
}

#[test]
fn flags_z_to_a_mixed_range() {
    // Z-a is uppercase start to lowercase end
    let src = "str.match?(/[Z-a]/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MixedCaseRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation for Z-a, got: {:?}", diags);
}
