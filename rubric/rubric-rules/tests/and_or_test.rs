use rubric_core::{LintContext, Rule};
use rubric_rules::style::and_or::AndOr;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/and_or/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/and_or/corrected.rb");

#[test]
fn detects_and_or_keywords() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AndOr.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/AndOr"));
}

#[test]
fn no_violation_for_double_ampersand_or_pipe() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = AndOr.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
