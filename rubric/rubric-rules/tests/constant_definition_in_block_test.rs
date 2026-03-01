use rubric_core::{LintContext, Rule};
use rubric_rules::lint::constant_definition_in_block::ConstantDefinitionInBlock;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/constant_definition_in_block/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/ConstantDefinitionInBlock"));
}

#[test]
fn no_violation_on_clean() {
    let src = "FOO = 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
