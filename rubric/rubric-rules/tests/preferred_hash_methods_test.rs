use rubric_core::{LintContext, Rule};
use rubric_rules::style::preferred_hash_methods::PreferredHashMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/preferred_hash_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = PreferredHashMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/PreferredHashMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "h.key?(:a)\nh.value?(1)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = PreferredHashMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
