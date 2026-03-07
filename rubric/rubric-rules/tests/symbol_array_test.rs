use rubric_core::{LintContext, Rule};
use rubric_rules::style::symbol_array::SymbolArray;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/symbol_array/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/symbol_array/corrected.rb");

#[test]
fn detects_symbol_array_that_should_use_percent_i() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SymbolArray.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for symbol array, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SymbolArray"));
}

#[test]
fn no_violation_with_percent_i_syntax() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SymbolArray.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// Symbol arrays in YARD/RDoc comment examples must not be flagged
#[test]
fn no_false_positive_for_symbol_array_in_comment() {
    let src = "#   Faker::Date.on_day(day: [:saturday, :sunday])\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SymbolArray.check_source(&ctx);
    assert!(diags.is_empty(), "symbol array in comment falsely flagged: {:?}", diags);
}
