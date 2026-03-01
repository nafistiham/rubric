use rubric_core::{LintContext, Rule};
use rubric_rules::lint::suppressed_exception::SuppressedException;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/suppressed_exception/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/suppressed_exception/corrected.rb");

#[test]
fn detects_suppressed_exception() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SuppressedException.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/SuppressedException"));
}

#[test]
fn no_violation_for_handled_exception() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SuppressedException.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
