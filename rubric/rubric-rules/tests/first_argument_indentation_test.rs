use rubric_core::{LintContext, Rule};
use rubric_rules::layout::first_argument_indentation::FirstArgumentIndentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/first_argument_indentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/FirstArgumentIndentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(\n  bar,\n  baz\n)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FirstArgumentIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
