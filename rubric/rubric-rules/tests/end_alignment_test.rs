use rubric_core::{LintContext, Rule};
use rubric_rules::layout::end_alignment::EndAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/end_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/end_alignment/corrected.rb");

#[test]
fn detects_misaligned_end() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EndAlignment"));
}

#[test]
fn no_violation_for_aligned_end() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
