use rubric_core::{LintContext, Rule};
use rubric_rules::style::even_odd::EvenOdd;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/even_odd/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/even_odd/clean.rb");

#[test]
fn detects_modulo_even_check() {
    let src = "x % 2 == 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `% 2 == 0`");
    assert!(diags[0].message.contains("even?"));
}

#[test]
fn detects_modulo_odd_check_ne_zero() {
    let src = "x % 2 != 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `% 2 != 0`");
    assert!(diags[0].message.contains("odd?"));
}

#[test]
fn detects_modulo_odd_check_eq_one() {
    let src = "x % 2 == 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `% 2 == 1`");
    assert!(diags[0].message.contains("odd?"));
}

#[test]
fn detects_modulo_even_check_ne_one() {
    let src = "x % 2 != 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for `% 2 != 1`");
    assert!(diags[0].message.contains("even?"));
}

#[test]
fn no_violation_for_even_predicate() {
    let src = "x.even?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_for_odd_predicate() {
    let src = "x.odd?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn skips_comment_lines() {
    let src = "# x % 2 == 0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_pattern_inside_string() {
    let src = "puts \"% 2 == 0\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EvenOdd.check_source(&ctx);
    assert!(diags.is_empty(), "string content should not be flagged, got: {:?}", diags);
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EvenOdd.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/EvenOdd"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = EvenOdd.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb, got: {:?}", diags);
}
