use rubric_core::{LintContext, Rule};
use rubric_rules::lint::symbol_conversion::SymbolConversion;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/symbol_conversion/offending.rb");
const PASSING: &str = include_str!("fixtures/lint/symbol_conversion/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SymbolConversion.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/SymbolConversion"));
}

#[test]
fn no_violation_on_clean_code() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = SymbolConversion.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}
