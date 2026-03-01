use rubric_core::{LintContext, Rule};
use rubric_rules::lint::rand_one::RandOne;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/rand_one/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RandOne.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/RandOne"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = rand(10)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RandOne.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
