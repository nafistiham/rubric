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
fn no_violation_on_single_blank_line() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLines.check_source(&ctx);
    assert!(diags.is_empty());
}
