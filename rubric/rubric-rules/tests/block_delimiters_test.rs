use rubric_core::{LintContext, Rule};
use rubric_rules::style::block_delimiters::BlockDelimiters;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/block_delimiters/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/block_delimiters/corrected.rb");

#[test]
fn detects_multiline_brace_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/BlockDelimiters"));
}

#[test]
fn no_violation_for_do_end_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = BlockDelimiters.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
