use rubric_core::{LintContext, Rule};
use rubric_rules::layout::heredoc_indentation::HeredocIndentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/heredoc_indentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/HeredocIndentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "text = <<~RUBY\n  properly_indented\nRUBY\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HeredocIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
