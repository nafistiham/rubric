use rubric_core::{LintContext, Rule};
use rubric_rules::style::double_negation::DoubleNegation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/double_negation/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/double_negation/clean.rb");

#[test]
fn detects_double_negation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DoubleNegation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/DoubleNegation"));
}

#[test]
fn no_violation_for_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = DoubleNegation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn does_not_flag_in_comment() {
    let src = "# !!value is a double negation\nfoo = true\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DoubleNegation.check_source(&ctx);
    assert!(diags.is_empty(), "!! in comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_in_string() {
    let src = "x = \"!!value\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DoubleNegation.check_source(&ctx);
    assert!(diags.is_empty(), "!! in string should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_inline_double_negation() {
    let src = "valid = !!active\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DoubleNegation.check_source(&ctx);
    assert!(!diags.is_empty(), "!! in expression should be flagged");
    assert!(diags[0].message.contains("double negation"));
}

#[test]
fn counts_correct_number_of_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DoubleNegation.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations, got {}", diags.len());
}
