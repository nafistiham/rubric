use rubric_core::{LintContext, Rule};
use rubric_rules::lint::no_return_in_begin_end_block::NoReturnInBeginEndBlock;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/no_return_in_begin_end_block/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NoReturnInBeginEndBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/NoReturnInBeginEndBlock"));
}

#[test]
fn no_violation_on_clean() {
    let src = "BEGIN { puts 'hello' }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NoReturnInBeginEndBlock.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
