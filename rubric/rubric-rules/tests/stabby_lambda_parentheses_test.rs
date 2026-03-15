use rubric_core::{LintContext, Rule};
use rubric_rules::style::stabby_lambda_parentheses::StabbyLambdaParentheses;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/stabby_lambda_parentheses/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/stabby_lambda_parentheses/passing.rb");

#[test]
fn detects_violations_in_offending() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(
        diags.iter().all(|d| d.rule == "Style/StabbyLambdaParentheses"),
        "unexpected rule names: {:?}",
        diags.iter().map(|d| d.rule).collect::<Vec<_>>()
    );
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_space_before_paren() {
    let src = "f = -> (x) { x + 1 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `-> (`");
    assert!(diags[0].message.contains("space"));
}

#[test]
fn does_not_flag_no_space_before_paren() {
    let src = "f = ->(x) { x + 1 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "`->(` must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_arrow_without_params() {
    let src = "f = -> { puts 'hi' }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "arrow without params must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# -> (x) { x }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "comment line must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_pattern_inside_string() {
    let src = "msg = \"-> (x) { x }\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "pattern inside string must not be flagged: {:?}", diags);
}

#[test]
fn counts_each_violation_on_same_line() {
    // Two separate lambdas on one line — unlikely in real code but tests count
    let src = "a = -> (x) { x }; b = -> (y) { y }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StabbyLambdaParentheses.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations, got: {:?}", diags);
}
