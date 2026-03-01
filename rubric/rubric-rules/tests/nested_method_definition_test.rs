use rubric_core::{LintContext, Rule};
use rubric_rules::lint::nested_method_definition::NestedMethodDefinition;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/nested_method_definition/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/NestedMethodDefinition"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  bar\nend\n\ndef bar\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NestedMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
