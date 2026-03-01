use rubric_core::{LintContext, Rule};
use rubric_rules::lint::flip_flop::FlipFlop;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/flip_flop/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FlipFlop.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/FlipFlop"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = 1..10\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FlipFlop.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
