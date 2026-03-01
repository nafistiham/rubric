use rubric_core::{LintContext, Rule};
use rubric_rules::style::symbol_proc::SymbolProc;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/symbol_proc/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/symbol_proc/corrected.rb");

#[test]
fn detects_expandable_symbol_proc() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SymbolProc.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SymbolProc"));
}

#[test]
fn no_violation_for_symbol_proc_shorthand() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SymbolProc.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
