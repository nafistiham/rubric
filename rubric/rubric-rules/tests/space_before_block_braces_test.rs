use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_before_block_braces::SpaceBeforeBlockBraces;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_before_block_braces/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_before_block_braces/corrected.rb");

#[test]
fn detects_missing_space_before_block_brace() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceBeforeBlockBraces.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceBeforeBlockBraces"));
}

#[test]
fn no_violation_with_space_before_block_brace() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceBeforeBlockBraces.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
