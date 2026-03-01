use rubric_core::{LintContext, Rule};
use rubric_rules::style::ternary_parentheses::TernaryParentheses;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/ternary_parentheses/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/ternary_parentheses/corrected.rb");

#[test]
fn detects_ternary_with_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/TernaryParentheses"));
}

#[test]
fn no_violation_for_ternary_without_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
