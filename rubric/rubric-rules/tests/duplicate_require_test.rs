use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_require::DuplicateRequire;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/duplicate_require/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateRequire"));
}

#[test]
fn no_violation_on_clean() {
    let src = "require 'foo'\nrequire 'bar'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateRequire.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
