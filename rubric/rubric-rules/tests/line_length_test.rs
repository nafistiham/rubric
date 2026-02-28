use rubric_core::{LintContext, Rule};
use rubric_rules::layout::line_length::LineLength;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/line_length/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/line_length/corrected.rb");

#[test]
fn detects_long_lines() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = LineLength.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/LineLength"));
}

#[test]
fn no_violation_on_short_lines() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = LineLength.check_source(&ctx);
    assert!(diags.is_empty());
}
