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

#[test]
fn no_false_positive_for_empty_parens_in_multiline_regex() {
    // [!$()*+,;=] inside a multiline /regex/ — $() is a regex char class, not empty expr
    let src = concat!(
        "VALIDATE = /(?:\n",
        "  #{STUFF}|\n",
        "  [!$()*+,;=]\n",
        ")/iox\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in multiline regex body, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_empty_parens_in_heredoc_body() {
    let src = concat!(
        "execute(<<-SQL)\n",
        "  SELECT * FROM t WHERE f() IS NOT NULL\n",
        "SQL\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in heredoc body, got: {:?}", diags);
}
