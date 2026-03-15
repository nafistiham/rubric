use rubric_core::{LintContext, Rule};
use rubric_rules::style::case_like_if::CaseLikeIf;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/case_like_if/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/case_like_if/clean.rb");

#[test]
fn detects_case_like_if_chain() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CaseLikeIf.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(
        diags.iter().all(|d| d.rule == "Style/CaseLikeIf"),
        "all diagnostics should be tagged correctly"
    );
}

#[test]
fn detects_correct_count() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CaseLikeIf.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations");
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = CaseLikeIf.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_violation_single_if_no_elsif() {
    let src = "if x == 1\n  do_one\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseLikeIf.check_source(&ctx);
    assert!(diags.is_empty(), "single if without elsif should not be flagged");
}

#[test]
fn no_violation_different_lhs() {
    let src = "if a == 1\n  first\nelsif b == 2\n  second\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseLikeIf.check_source(&ctx);
    assert!(diags.is_empty(), "different LHS variables should not be flagged");
}

#[test]
fn detects_with_two_branches_same_lhs() {
    let src = "if color == :red\n  stop\nelsif color == :green\n  go\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseLikeIf.check_source(&ctx);
    assert_eq!(diags.len(), 1, "two branches with same LHS should be flagged");
}
