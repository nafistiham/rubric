use rubric_core::{LintContext, Rule};
use rubric_rules::style::lambda::Lambda;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/lambda/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/lambda/corrected.rb");

#[test]
fn detects_lambda_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Lambda.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/Lambda"));
}

#[test]
fn no_violation_for_stabby_lambda() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = Lambda.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
