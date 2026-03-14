use rubric_core::{LintContext, Rule};
use rubric_rules::lint::inherit_exception::InheritException;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/inherit_exception/offending.rb");
const CLEAN: &str = include_str!("fixtures/lint/inherit_exception/clean.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = InheritException.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/InheritException"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = InheritException.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_bare_exception() {
    let src = "class MyError < Exception\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InheritException.check_source(&ctx);
    assert!(!diags.is_empty(), "bare Exception should be flagged");
    assert_eq!(diags[0].rule, "Lint/InheritException");
    assert!(diags[0].message.contains("StandardError"));
}

#[test]
fn detects_namespaced_exception() {
    let src = "class MyError < ::Exception\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InheritException.check_source(&ctx);
    assert!(!diags.is_empty(), "::Exception should be flagged");
}

#[test]
fn does_not_flag_standard_error() {
    let src = "class MyError < StandardError\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InheritException.check_source(&ctx);
    assert!(diags.is_empty(), "StandardError should not be flagged");
}

#[test]
fn does_not_flag_runtime_error() {
    let src = "class MyError < RuntimeError\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InheritException.check_source(&ctx);
    assert!(diags.is_empty(), "RuntimeError should not be flagged");
}

#[test]
fn does_not_flag_comment() {
    let src = "# class BadError < Exception is wrong\nclass GoodError < StandardError\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InheritException.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_exception_subclass_name() {
    // ExceptionHandler is NOT inheriting from Exception
    let src = "class ExceptionHandler < StandardError\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InheritException.check_source(&ctx);
    assert!(diags.is_empty(), "ExceptionHandler parent should not be flagged");
}
