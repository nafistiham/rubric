use rubric_core::{LintContext, Rule};
use rubric_rules::layout::indentation_style::IndentationStyle;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/indentation_style/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IndentationStyle.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/IndentationStyle"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationStyle.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
