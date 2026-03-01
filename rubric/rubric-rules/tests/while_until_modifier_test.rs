use rubric_core::{LintContext, Rule};
use rubric_rules::style::while_until_modifier::WhileUntilModifier;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/while_until_modifier/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/while_until_modifier/corrected.rb");

#[test]
fn detects_while_block_that_could_be_modifier() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = WhileUntilModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/WhileUntilModifier"));
}

#[test]
fn no_violation_for_modifier_while() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = WhileUntilModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
