use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_line_after_guard_clause::EmptyLineAfterGuardClause;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_line_after_guard_clause/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/layout/empty_line_after_guard_clause/clean.rb");

#[test]
fn detects_missing_empty_line_after_guard_clause() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLineAfterGuardClause.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLineAfterGuardClause"));
}

#[test]
fn no_violation_when_empty_line_after_guard_clause() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = EmptyLineAfterGuardClause.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_when_guard_clause_before_end() {
    let src = "def foo(x)\n  return if x.nil?\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLineAfterGuardClause.check_source(&ctx);
    assert!(diags.is_empty(), "guard clause before end should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_when_guard_clause_before_else() {
    let src = "def foo(x)\n  if x\n    return if x.nil?\n  else\n    1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLineAfterGuardClause.check_source(&ctx);
    assert!(diags.is_empty(), "guard clause before else should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_raise_as_guard_clause() {
    let src = "def foo(x)\n  raise ArgumentError if x.nil?\n  do_something\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLineAfterGuardClause.check_source(&ctx);
    assert!(!diags.is_empty(), "raise guard clause should be flagged");
}
