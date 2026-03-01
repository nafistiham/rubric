use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_line_between_defs::EmptyLineBetweenDefs;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_line_between_defs/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/empty_line_between_defs/corrected.rb");

#[test]
fn detects_missing_empty_line_between_defs() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLineBetweenDefs.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLineBetweenDefs"));
}

#[test]
fn no_violation_with_blank_line_between_defs() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLineBetweenDefs.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
