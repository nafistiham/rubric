use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_definition_brace_layout::MultilineMethodDefinitionBraceLayout;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/multiline_method_definition_brace_layout/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodDefinitionBraceLayout"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo(\n  bar,\n  baz\n)\n  bar + baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultilineMethodDefinitionBraceLayout.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
