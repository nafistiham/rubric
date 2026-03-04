use rubric_core::{LintContext, Rule};
use rubric_rules::style::ternary_parentheses::TernaryParentheses;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/ternary_parentheses/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/ternary_parentheses/corrected.rb");

#[test]
fn detects_ternary_with_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/TernaryParentheses"));
}

#[test]
fn no_violation_for_ternary_without_parens() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// FP: is_a?(Type) ? — the `(` opens a method argument list, not a ternary condition wrapper
#[test]
fn no_false_positive_for_is_a_predicate_method_before_ternary() {
    let src = "x = obj.is_a?(String) ? 'yes' : 'no'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for is_a? method call, got: {:?}", diags);
}

// FP: respond_to?(:sym) ? — same pattern, predicate method with symbol arg
#[test]
fn no_false_positive_for_respond_to_predicate_method_before_ternary() {
    let src = "sym = k.respond_to?(:to_sym) ? k.to_sym : k\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for respond_to? method call, got: {:?}", diags);
}

// FP: method?(arg) ? — generic predicate method with any identifier arg
#[test]
fn no_false_positive_for_generic_predicate_method_before_ternary() {
    let src = "x = check?(Proc) ? a : b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for predicate method call, got: {:?}", diags);
}

// Real violation: (comparison) ? — space before `(` means it wraps the condition
#[test]
fn still_detects_unnecessary_parens_around_comparison() {
    let src = "x = (a > b) ? 'yes' : 'no'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for (a > b) ?, got none");
}

// Real violation: (Hash === v) ?
#[test]
fn still_detects_unnecessary_parens_around_case_equality() {
    let src = "result = (Hash === v) ? x : y\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for (Hash === v) ?, got none");
}

// Real violation: (pageidx.to_i < 1) ?
#[test]
fn still_detects_unnecessary_parens_around_method_call_comparison() {
    let src = "p = (pageidx.to_i < 1) ? 1 : pageidx.to_i\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for (pageidx.to_i < 1) ?, got none");
}

// FP: AllowSafeAssignment — `(var = expr) ?` parens are required to make assignment the condition
#[test]
fn no_false_positive_for_safe_assignment_in_ternary_condition() {
    let src = "result = (x = foo) ? x.bar : default\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for (x = foo) ? (safe assignment), got: {:?}", diags);
}

// FP: assignment with method call RHS — still a plain `=`
#[test]
fn no_false_positive_for_safe_assignment_with_method_rhs() {
    let src = "value = (chunk = body.read(LEN)) ? process(chunk) : nil\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TernaryParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for (chunk = body.read(LEN)) ? (safe assignment), got: {:?}", diags);
}
