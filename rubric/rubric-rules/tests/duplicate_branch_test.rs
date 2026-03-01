use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_branch::DuplicateBranch;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_branch/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateBranch.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateBranch"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if foo\n  bar\nelse\n  baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateBranch.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
