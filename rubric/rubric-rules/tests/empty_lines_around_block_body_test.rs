use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines_around_block_body::EmptyLinesAroundBlockBody;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/empty_lines_around_block_body/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLinesAroundBlockBody.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLinesAroundBlockBody"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2, 3].each do |x|\n  puts x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundBlockBody.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
