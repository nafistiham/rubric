use rubric_core::{LintContext, Rule};
use rubric_rules::lint::empty_conditional_body::EmptyConditionalBody;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/empty_conditional_body/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyConditionalBody.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/EmptyConditionalBody"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if foo\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyConditionalBody.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
