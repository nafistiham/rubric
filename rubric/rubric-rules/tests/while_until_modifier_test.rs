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

// FP: `while chunk = body.read(LEN)` — the assignment is the condition, cannot use modifier form
#[test]
fn no_false_positive_for_while_with_assignment_condition() {
    let src = "while chunk = body.read(LEN)\n  process(chunk)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhileUntilModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for assignment condition, got: {:?}", diags);
}

// FP: `while line = gets` — idiomatic Ruby input reading
#[test]
fn no_false_positive_for_while_line_gets() {
    let src = "while line = gets\n  puts line\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhileUntilModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for `while line = gets`, got: {:?}", diags);
}

// FP: `while neg_data = @engine.extract` — assignment condition with instance variable method
#[test]
fn no_false_positive_for_while_ivar_method_assignment_condition() {
    let src = "while neg_data = @engine.extract\n  results << neg_data\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhileUntilModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation for ivar method assignment condition, got: {:?}", diags);
}
