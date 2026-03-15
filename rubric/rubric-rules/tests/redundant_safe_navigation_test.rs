use rubric_core::{LintContext, Rule};
use rubric_rules::lint::redundant_safe_navigation::RedundantSafeNavigation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/redundant_safe_navigation/offending.rb");
const CORRECTED: &str = include_str!("fixtures/lint/redundant_safe_navigation/corrected.rb");

#[test]
fn detects_redundant_safe_navigation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/RedundantSafeNavigation"));
}

#[test]
fn detects_all_four_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert_eq!(diags.len(), 4, "expected 4 violations, got: {:?}", diags);
}

#[test]
fn no_violation_on_corrected() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on corrected code, got: {:?}", diags);
}

#[test]
fn detects_string_literal_double_quote() {
    let src = "\"hello\"&.upcase\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for double-quoted string literal");
}

#[test]
fn detects_string_literal_single_quote() {
    let src = "'hello'&.upcase\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for single-quoted string literal");
}

#[test]
fn detects_array_literal() {
    let src = "[1, 2, 3]&.first\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for array literal");
}

#[test]
fn detects_hash_literal() {
    let src = "{a: 1}&.keys\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for hash literal");
}

#[test]
fn detects_integer_literal() {
    let src = "42&.to_s\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for integer literal");
}

#[test]
fn no_violation_for_variable() {
    let src = "str&.upcase\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for variable receiver, got: {:?}", diags);
}

#[test]
fn no_violation_in_comment() {
    let src = "# \"string\"&.upcase is bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation in comment, got: {:?}", diags);
}
