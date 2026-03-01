use rubric_core::{LintContext, Rule};
use rubric_rules::lint::big_decimal_new::BigDecimalNew;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/big_decimal_new/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BigDecimalNew.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/BigDecimalNew"));
}

#[test]
fn no_violation_on_clean() {
    let src = "x = BigDecimal('1.0')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BigDecimalNew.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
