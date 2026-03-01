use rubric_core::{LintContext, Rule};
use rubric_rules::style::trailing_underscore_variable::TrailingUnderscoreVariable;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/trailing_underscore_variable/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrailingUnderscoreVariable.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/TrailingUnderscoreVariable"));
}

#[test]
fn no_violation_on_clean() {
    let src = "a, b = foo\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrailingUnderscoreVariable.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
