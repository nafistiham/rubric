use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unused_block_argument::UnusedBlockArgument;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/unused_block_argument/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnusedBlockArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnusedBlockArgument"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2].each do |x|\n  puts x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedBlockArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
