use rubric_core::{LintContext, Rule};
use rubric_rules::layout::trailing_newlines::TrailingNewlines;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/trailing_newlines/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/trailing_newlines/corrected.rb");

#[test]
fn detects_multiple_trailing_newlines() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrailingNewlines.check_source(&ctx);
    assert!(!diags.is_empty(), "expected at least one violation");
    assert!(diags.iter().all(|d| d.rule == "Layout/TrailingEmptyLines"));
}

#[test]
fn no_violation_on_single_trailing_newline() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = TrailingNewlines.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations");
}
