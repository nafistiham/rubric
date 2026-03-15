use rubric_core::{LintContext, Rule};
use rubric_rules::style::combined_comparison::CombinedComparison;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/combined_comparison/offending.rb");
const PASSING: &str = include_str!("fixtures/style/combined_comparison/passing.rb");

#[test]
fn detects_violations_in_offending() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(
        diags.iter().all(|d| d.rule == "Style/CombinedComparison"),
        "unexpected rule names: {:?}",
        diags.iter().map(|d| d.rule).collect::<Vec<_>>()
    );
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_inline_ternary_comparison_chain() {
    let src = "a > b ? 1 : a < b ? -1 : 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for ternary comparison chain");
    assert!(diags[0].message.contains("<=>"));
}

#[test]
fn detects_parenthesised_variant() {
    let src = "a > b ? 1 : (a < b ? -1 : 0)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for parenthesised variant");
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# a > b ? 1 : a < b ? -1 : 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(diags.is_empty(), "comment line must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_simple_ternary() {
    let src = "result = a > b ? \"bigger\" : \"smaller\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(diags.is_empty(), "simple ternary must not be flagged: {:?}", diags);
}

#[test]
fn does_not_flag_spaceship_usage() {
    let src = "a <=> b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CombinedComparison.check_source(&ctx);
    assert!(diags.is_empty(), "spaceship operator itself must not be flagged: {:?}", diags);
}
