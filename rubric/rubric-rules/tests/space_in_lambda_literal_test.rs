use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_in_lambda_literal::SpaceInLambdaLiteral;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/space_in_lambda_literal/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInLambdaLiteral.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInLambdaLiteral"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo = ->(x) { x + 1 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceInLambdaLiteral.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
