use rubric_core::{LintContext, Rule};
use rubric_rules::layout::closing_parenthesis_indentation::ClosingParenthesisIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/closing_parenthesis_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/closing_parenthesis_indentation/corrected.rb");

#[test]
fn detects_misindented_closing_paren() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClosingParenthesisIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for odd-indented `)`, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/ClosingParenthesisIndentation"));
}

#[test]
fn no_violation_with_correct_closing_paren_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = ClosingParenthesisIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
