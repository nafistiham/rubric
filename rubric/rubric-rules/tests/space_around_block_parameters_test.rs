use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_block_parameters::SpaceAroundBlockParameters;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_around_block_parameters/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_around_block_parameters/corrected.rb");

#[test]
fn detects_missing_space_around_block_parameters() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundBlockParameters.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `{{|x|`, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundBlockParameters"));
}

#[test]
fn no_violation_with_space_around_block_parameters() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAroundBlockParameters.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
