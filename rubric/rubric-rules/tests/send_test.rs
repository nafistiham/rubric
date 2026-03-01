use rubric_core::{LintContext, Rule};
use rubric_rules::style::send::Send;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/send/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Send.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/Send"));
}

#[test]
fn no_violation_on_clean() {
    let src = "obj.public_send(:foo)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Send.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
