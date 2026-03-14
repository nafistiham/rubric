use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/style/empty_else.rs"]
mod empty_else;
use empty_else::EmptyElse;

const OFFENDING: &str = include_str!("fixtures/style/empty_else/offending.rb");
const PASSING: &str = include_str!("fixtures/style/empty_else/passing.rb");

#[test]
fn detects_empty_else() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyElse.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/EmptyElse"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = EmptyElse.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_else_followed_by_end() {
    let src = "if x\n  foo\nelse\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyElse.check_source(&ctx);
    assert!(!diags.is_empty(), "else..end should be flagged");
    assert_eq!(diags[0].rule, "Style/EmptyElse");
}

#[test]
fn detects_else_followed_by_nil() {
    let src = "if x\n  foo\nelse\n  nil\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyElse.check_source(&ctx);
    assert!(!diags.is_empty(), "else..nil should be flagged");
}

#[test]
fn does_not_flag_else_with_body() {
    let src = "if x\n  foo\nelse\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyElse.check_source(&ctx);
    assert!(diags.is_empty(), "else with body should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_if_without_else() {
    let src = "if x\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyElse.check_source(&ctx);
    assert!(diags.is_empty(), "if without else should not be flagged, got: {:?}", diags);
}

#[test]
fn correct_message() {
    let src = "if x\n  foo\nelse\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyElse.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("empty"),
        "message should mention empty, got: {}",
        diags[0].message
    );
}
