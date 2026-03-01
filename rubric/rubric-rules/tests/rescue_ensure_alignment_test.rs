use rubric_core::{LintContext, Rule};
use rubric_rules::layout::rescue_ensure_alignment::RescueEnsureAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/rescue_ensure_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/rescue_ensure_alignment/corrected.rb");

#[test]
fn detects_misaligned_rescue() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/RescueEnsureAlignment"));
}

#[test]
fn no_violation_for_aligned_rescue() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
