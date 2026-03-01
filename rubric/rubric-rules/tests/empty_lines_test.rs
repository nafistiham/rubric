use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines::EmptyLines;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/empty_lines/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/empty_lines/corrected.rb");

#[test]
fn detects_multiple_blank_lines() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLines.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLines"));
}

#[test]
fn each_excess_blank_line_reported() {
    // Three consecutive blank lines should produce two violations
    // (one for blank_run==2, one for blank_run==3)
    let source = "def foo\nend\n\n\n\ndef bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = EmptyLines.check_source(&ctx);
    assert_eq!(diags.len(), 2, "three blank lines should produce two violations, got: {:?}", diags);
}

#[test]
fn no_violation_on_single_blank_line() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLines.check_source(&ctx);
    assert!(diags.is_empty());
}
