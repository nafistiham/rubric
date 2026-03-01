use rubric_core::{LintContext, Rule};
use rubric_rules::layout::else_alignment::ElseAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/else_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/ElseAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if foo\n  bar\nelse\n  baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
