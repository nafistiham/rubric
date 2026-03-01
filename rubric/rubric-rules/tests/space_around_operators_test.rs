use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_operators::SpaceAroundOperators;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_around_operators/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_around_operators/corrected.rb");

#[test]
fn detects_missing_space_around_operators() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundOperators"));
}

#[test]
fn no_violation_with_spaces_around_operators() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
