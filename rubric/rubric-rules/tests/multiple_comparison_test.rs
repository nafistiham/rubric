use rubric_core::{LintContext, Rule};
use rubric_rules::lint::multiple_comparison::MultipleComparison;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/multiple_comparison/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/MultipleComparison"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if x > 1 && x < 10\n  puts 'in range'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MultipleComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
