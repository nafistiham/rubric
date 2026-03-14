use rubric_core::{LintContext, Rule};
use rubric_rules::lint::percent_string_array::PercentStringArray;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/percent_string_array/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/percent_string_array/clean.rb");

#[test]
fn detects_comma_in_percent_w() {
    let src = "words = %w[foo, bar]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PercentStringArray.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for comma in %w");
    assert!(diags[0].message.contains("separator"));
}

#[test]
fn detects_comma_in_percent_w_uppercase() {
    let src = "words = %W[foo, bar]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PercentStringArray.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for comma in %W");
}

#[test]
fn no_violation_clean_percent_w() {
    let src = "words = %w[foo bar baz]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PercentStringArray.check_source(&ctx);
    assert!(diags.is_empty(), "clean %w should not be flagged");
}

#[test]
fn no_violation_empty_percent_w() {
    let src = "words = %w[]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PercentStringArray.check_source(&ctx);
    assert!(diags.is_empty(), "empty %w should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = PercentStringArray.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/PercentStringArray"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = PercentStringArray.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}
