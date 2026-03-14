use rubric_core::{LintContext, Rule};
use rubric_rules::naming::method_name::MethodName;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/naming/method_name/offending.rb");
const CLEAN: &str = include_str!("fixtures/naming/method_name/clean.rb");

#[test]
fn detects_camel_case_method() {
    let src = "def myMethod\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for myMethod");
    assert!(diags[0].message.contains("snake_case"));
}

#[test]
fn detects_pascal_case_method() {
    let src = "def DoSomething\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for DoSomething");
}

#[test]
fn no_violation_snake_case() {
    let src = "def my_method\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodName.check_source(&ctx);
    assert!(diags.is_empty(), "snake_case method should not be flagged");
}

#[test]
fn no_violation_predicate_method() {
    let src = "def valid?(x)\n  x > 0\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodName.check_source(&ctx);
    assert!(diags.is_empty(), "predicate method should not be flagged");
}

#[test]
fn no_violation_operator() {
    let src = "def ==(other)\n  super\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MethodName.check_source(&ctx);
    assert!(diags.is_empty(), "operator method should not be flagged");
}

#[test]
fn offending_fixture_has_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MethodName.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Naming/MethodName"));
}

#[test]
fn clean_fixture_has_no_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = MethodName.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in clean.rb");
}
