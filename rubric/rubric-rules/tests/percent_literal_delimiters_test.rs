use rubric_core::{LintContext, Rule};
use rubric_rules::style::percent_literal_delimiters::PercentLiteralDelimiters;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/percent_literal_delimiters/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = PercentLiteralDelimiters.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/PercentLiteralDelimiters"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = %w[foo bar]\ny = %i[one two]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PercentLiteralDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
