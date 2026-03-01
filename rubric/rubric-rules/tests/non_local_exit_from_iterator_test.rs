use rubric_core::{LintContext, Rule};
use rubric_rules::lint::non_local_exit_from_iterator::NonLocalExitFromIterator;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/non_local_exit_from_iterator/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/NonLocalExitFromIterator"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2].each do |x|\n  next if x > 1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
