use rubric_core::{LintContext, Rule};
use rubric_rules::style::numeric_predicate::NumericPredicate;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/numeric_predicate/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/numeric_predicate/clean.rb");

#[test]
fn detects_eq_zero() {
    let src = "x == 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `x == 0`");
    assert!(diags[0].message.contains("zero?"));
}

#[test]
fn detects_gt_zero() {
    let src = "x > 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `x > 0`");
    assert!(diags[0].message.contains("positive?"));
}

#[test]
fn detects_lt_zero() {
    let src = "x < 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `x < 0`");
    assert!(diags[0].message.contains("negative?"));
}

#[test]
fn no_violation_on_predicate_style() {
    let src = "x.zero?\ny.positive?\nz.negative?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(diags.is_empty(), "predicate style should not be flagged");
}

#[test]
fn no_violation_in_string() {
    let src = "msg = \"x == 0\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(diags.is_empty(), "pattern inside string should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/NumericPredicate"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = NumericPredicate.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}
