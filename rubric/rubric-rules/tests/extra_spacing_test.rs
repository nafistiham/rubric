use rubric_core::{LintContext, Rule};
use rubric_rules::layout::extra_spacing::ExtraSpacing;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/extra_spacing/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/extra_spacing/corrected.rb");

#[test]
fn detects_extra_spacing() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/ExtraSpacing"));
}

#[test]
fn no_violation_for_single_spacing() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
