use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ambiguous_operator::AmbiguousOperator;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/ambiguous_operator/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/ambiguous_operator/corrected.rb");

#[test]
fn detects_ambiguous_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AmbiguousOperator.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/AmbiguousOperator"));
}

#[test]
fn no_violation_for_unambiguous_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = AmbiguousOperator.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
