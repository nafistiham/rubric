use rubric_core::{LintContext, Rule};
use rubric_rules::layout::indentation_consistency::IndentationConsistency;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/indentation_consistency/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/indentation_consistency/corrected.rb");

#[test]
fn detects_mixed_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IndentationConsistency.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/IndentationConsistency"));
}

#[test]
fn no_violation_for_consistent_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = IndentationConsistency.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
