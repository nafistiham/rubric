use rubric_core::{LintContext, Rule};
use rubric_rules::layout::block_alignment::BlockAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/block_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/BlockAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo do\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
