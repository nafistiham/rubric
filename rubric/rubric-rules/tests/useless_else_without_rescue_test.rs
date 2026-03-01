use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_else_without_rescue::UselessElseWithoutRescue;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/useless_else_without_rescue/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessElseWithoutRescue.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessElseWithoutRescue"));
}

#[test]
fn no_violation_on_clean() {
    let src = "begin\n  foo\nrescue => e\n  handle(e)\nelse\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessElseWithoutRescue.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
