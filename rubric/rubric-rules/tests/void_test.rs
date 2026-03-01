use rubric_core::{LintContext, Rule};
use rubric_rules::lint::void::Void;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/void/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Void.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/Void"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1\ny = x + 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Void.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
