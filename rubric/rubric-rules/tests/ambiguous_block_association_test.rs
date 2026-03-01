use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ambiguous_block_association::AmbiguousBlockAssociation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/ambiguous_block_association/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/ambiguous_block_association/corrected.rb");

#[test]
fn detects_ambiguous_block_association() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AmbiguousBlockAssociation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/AmbiguousBlockAssociation"));
}

#[test]
fn no_violation_for_unambiguous_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = AmbiguousBlockAssociation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
