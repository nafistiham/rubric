use rubric_core::{LintContext, Rule};
use rubric_rules::lint::boolean_symbol::BooleanSymbol;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/boolean_symbol/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BooleanSymbol.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/BooleanSymbol"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = true\ny = false\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BooleanSymbol.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
