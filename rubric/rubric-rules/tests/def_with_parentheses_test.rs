use rubric_core::{LintContext, Rule};
use rubric_rules::style::def_with_parentheses::DefWithParentheses;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/def_with_parentheses/offending.rb");
const PASSING: &str =
    include_str!("fixtures/style/def_with_parentheses/passing.rb");

#[test]
fn detects_def_with_empty_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for def with empty parens, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/DefWithParentheses"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_simple_def_empty_parens() {
    let src = "def foo()\n  42\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "def foo() should be flagged");
}

#[test]
fn no_violation_for_def_with_params() {
    let src = "def foo(x)\n  x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "def with params should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_def_no_parens() {
    let src = "def foo\n  42\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "def without parens should not be flagged");
}

#[test]
fn detects_predicate_def_empty_parens() {
    let src = "def valid?()\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "def valid?() should be flagged");
}

#[test]
fn detects_bang_def_empty_parens() {
    let src = "def save!()\n  persist!\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "def save!() should be flagged");
}

#[test]
fn no_violation_for_comment() {
    let src = "# def foo()\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefWithParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged");
}

#[test]
fn detects_exactly_four_violations_in_offending() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DefWithParentheses.check_source(&ctx);
    assert_eq!(diags.len(), 4, "expected 4 violations, got: {:?}", diags);
}
