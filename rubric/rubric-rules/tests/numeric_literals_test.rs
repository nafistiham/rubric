use rubric_core::{LintContext, Rule};
use rubric_rules::style::numeric_literals::NumericLiterals;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/numeric_literals/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/numeric_literals/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NumericLiterals.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/NumericLiterals"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = NumericLiterals.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on passing.rb; got: {:?}", diags);
}

#[test]
fn detects_five_digit_number() {
    let src = "n = 12345\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericLiterals.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for 12345");
}

#[test]
fn no_violation_for_four_digit_number() {
    let src = "n = 1234\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericLiterals.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for 1234; got: {:?}", diags);
}
