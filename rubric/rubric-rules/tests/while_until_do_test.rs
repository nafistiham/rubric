use rubric_core::{LintContext, Rule};
use rubric_rules::style::while_until_do::WhileUntilDo;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/while_until_do/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/while_until_do/corrected.rb");

#[test]
fn detects_redundant_do_in_while() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = WhileUntilDo.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/WhileUntilDo"));
}

#[test]
fn no_violation_for_while_without_do() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = WhileUntilDo.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
