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
    // Array/hash literals are excluded (can't distinguish from subscript access)
    // so only string and integer literals are detected — 2 of 4 offending lines
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations (string + integer), got: {:?}", diags);
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
fn no_violation_for_array_subscript() {
    // `arr[k]&.method` — subscript can return nil, not flagged
    // (can't distinguish from `[literal]` without AST, so both are skipped)
    let src = "arr[0]&.upcase\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for array subscript, got: {:?}", diags);
}

#[test]
fn no_violation_for_hash_subscript() {
    // `hash[:k]&.method` — subscript can return nil, not flagged
    let src = "hash[:a]&.keys\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSafeNavigation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for hash subscript, got: {:?}", diags);
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
