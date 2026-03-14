use rubric_core::{LintContext, Rule};
use rubric_rules::style::method_def_parentheses::MethodDefParentheses;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/method_def_parentheses/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/method_def_parentheses/clean.rb");

#[test]
fn detects_empty_parens_on_no_param_method() {
    let src = "def foo()\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for def foo()");
    assert!(diags[0].message.contains("Omit"));
}

#[test]
fn detects_params_without_parens() {
    let src = "def bar a, b\n  a + b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for params without parens");
    assert!(diags[0].message.contains("Use parentheses"));
}

#[test]
fn no_violation_no_params_no_parens() {
    let src = "def foo\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "no params + no parens should be clean");
}

#[test]
fn no_violation_params_with_parens() {
    let src = "def bar(a, b)\n  a + b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "params with parens should be clean");
}

#[test]
fn no_violation_operator_methods() {
    let src = "def ==(other)\n  super\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "operator methods should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/MethodDefParentheses"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = MethodDefParentheses.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}
