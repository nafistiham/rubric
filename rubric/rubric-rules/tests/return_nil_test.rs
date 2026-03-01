use rubric_core::{LintContext, Rule};
use rubric_rules::style::return_nil::ReturnNil;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/return_nil/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ReturnNil.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ReturnNil"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  return if something\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ReturnNil.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
