use rubric_core::{LintContext, Rule};
use rubric_rules::lint::raise_exception::RaiseException;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/raise_exception/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RaiseException.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/RaiseException"));
}

#[test]
fn no_violation_on_clean() {
    let src = "raise StandardError, 'error'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RaiseException.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
