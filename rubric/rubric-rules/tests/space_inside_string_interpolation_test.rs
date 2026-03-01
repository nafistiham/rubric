use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_string_interpolation::SpaceInsideStringInterpolation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_inside_string_interpolation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_inside_string_interpolation/corrected.rb");

#[test]
fn detects_space_inside_interpolation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideStringInterpolation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideStringInterpolation"));
}

#[test]
fn no_violation_for_clean_interpolation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceInsideStringInterpolation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
