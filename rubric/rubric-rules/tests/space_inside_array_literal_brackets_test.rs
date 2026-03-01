use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_inside_array_literal_brackets::SpaceInsideArrayLiteralBrackets;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_inside_array_literal_brackets/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_inside_array_literal_brackets/corrected.rb");

#[test]
fn detects_space_inside_array_literal_brackets() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceInsideArrayLiteralBrackets.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceInsideArrayLiteralBrackets"));
}

#[test]
fn no_violation_without_space_inside_array_literal_brackets() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceInsideArrayLiteralBrackets.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
