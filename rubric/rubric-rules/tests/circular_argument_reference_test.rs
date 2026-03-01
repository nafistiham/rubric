use rubric_core::{LintContext, Rule};
use rubric_rules::lint::circular_argument_reference::CircularArgumentReference;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/circular_argument_reference/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CircularArgumentReference.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/CircularArgumentReference"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo(bar = 1)\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CircularArgumentReference.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
