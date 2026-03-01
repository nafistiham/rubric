use rubric_core::{LintContext, Rule};
use rubric_rules::style::negated_while::NegatedWhile;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/negated_while/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NegatedWhile.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedWhile"));
}

#[test]
fn no_violation_on_clean() {
    let src = "until condition\n  do_something\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedWhile.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
