use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_methods::DuplicateMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  1\nend\n\ndef bar\n  2\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
