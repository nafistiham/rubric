use rubric_core::{LintContext, Rule};
use rubric_rules::lint::parentheses_as_grouped_expression::ParenthesesAsGroupedExpression;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/parentheses_as_grouped_expression/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/ParenthesesAsGroupedExpression"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(bar)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Keyword guards

#[test]
fn no_violation_for_elsif_condition() {
    // `elsif (var = expr)` is a control-flow keyword, not a method call
    let src = "if x\n  y\nelsif (var = something)\n  z\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for elsif (...): {:?}",
        diags
    );
}

#[test]
fn no_violation_for_if_condition() {
    let src = "if (x > 0)\n  y\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for if (...)");
}

#[test]
fn no_violation_for_while_condition() {
    let src = "while (condition)\n  break\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for while (...)");
}

#[test]
fn no_violation_for_return_grouped() {
    let src = "def foo\n  return (value)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for return (...)");
}

#[test]
fn no_violation_for_rescue_keyword() {
    let src = "begin\n  x\nrescue (RuntimeError)\n  nil\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for rescue (...)");
}

#[test]
fn no_violation_for_when_clause() {
    let src = "case x\nwhen (1..5)\n  y\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for when (...)");
}

// Heredoc body skipping

#[test]
fn no_violation_inside_heredoc_body() {
    // SQL inside a squiggly heredoc - FROM ( and WHERE (( must not be flagged
    let src = "def query\n  <<~SQL\n    SELECT count(*) FROM (\n      SELECT id FROM users\n      WHERE (users.active = TRUE)\n    ) AS subquery\n  SQL\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations inside heredoc body: {:?}",
        diags
    );
}

#[test]
fn no_violation_inside_dash_heredoc_body() {
    // <<-SQL variant used in schema.rb / migration_helpers.rb
    let src = "raise <<-EOF\ndatabase (dbname) using a super user\nEOF\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations inside <<-EOF heredoc body: {:?}",
        diags
    );
}

#[test]
fn no_violation_inside_squish_heredoc() {
    // <<~SQL.squish pattern from mastodon source.rb / measure files
    let src = "def follows_sql\n  <<~SQL.squish\n    EXISTS (SELECT 1 FROM follows WHERE follows.id = :id)\n  SQL\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations inside squish heredoc: {:?}",
        diags
    );
}

// Multiple-arg grouped expression guard

#[test]
fn no_violation_when_grouped_expr_is_first_of_multiple_args() {
    // link_to (expr || other), path - (expr) is grouped arg, not method paren
    let src = "link_to (log.presence || default), admin_path(id)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for link_to (grouped), other: {:?}",
        diags
    );
}

// Real violations still detected

#[test]
fn still_detects_single_arg_method_space_paren() {
    let src = "foo (bar)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation for foo (bar)");
}

#[test]
fn still_detects_puts_space_paren() {
    let src = "puts (x + 1)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ParenthesesAsGroupedExpression.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation for puts (x + 1)");
}
