use rubric_core::{LintContext, Rule};
use rubric_rules::lint::empty_expression::EmptyExpression;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/empty_expression/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/empty_expression/corrected.rb");

#[test]
fn detects_empty_expression() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyExpression.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/EmptyExpression"));
}

#[test]
fn no_violation_for_non_empty_expression() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
