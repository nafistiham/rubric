use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ensure_return::EnsureReturn;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/ensure_return/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EnsureReturn.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/EnsureReturn"));
}

#[test]
fn no_violation_on_clean() {
    let src = "begin\n  foo\nensure\n  cleanup\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EnsureReturn.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
