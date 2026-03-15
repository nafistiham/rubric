use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_capital_w::RedundantCapitalW;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/redundant_capital_w/offending.rb");
const PASSING: &str = include_str!("fixtures/style/redundant_capital_w/passing.rb");

#[test]
fn detects_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantCapitalW"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}

#[test]
fn flags_percent_w_without_interpolation() {
    let src = "arr = %W[foo bar]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for %W without interpolation");
}

#[test]
fn does_not_flag_percent_w_with_interpolation() {
    let src = "arr = %W[foo #{name} bar]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(diags.is_empty(), "should not flag %W with interpolation, got: {:?}", diags);
}

#[test]
fn does_not_flag_lowercase_percent_w() {
    let src = "arr = %w[foo bar baz]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(diags.is_empty(), "%w is fine, got: {:?}", diags);
}

#[test]
fn flags_percent_w_with_parens() {
    let src = "arr = %W(one two three)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for %W( without interpolation");
}

#[test]
fn message_mentions_percent_w() {
    let src = "arr = %W[foo bar]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantCapitalW.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("%w"), "message should mention %w");
}
