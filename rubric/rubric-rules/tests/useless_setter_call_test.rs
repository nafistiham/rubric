use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_setter_call::UselessSetterCall;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/useless_setter_call/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessSetterCall"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  self.bar = 1\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessSetterCall.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
