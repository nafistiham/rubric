use rubric_core::{LintContext, Rule};
use rubric_rules::lint::empty_block::EmptyBlock;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/empty_block/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/empty_block/corrected.rb");

#[test]
fn detects_empty_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/EmptyBlock"));
}

#[test]
fn no_violation_for_non_empty_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyBlock.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
