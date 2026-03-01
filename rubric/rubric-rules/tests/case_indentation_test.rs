use rubric_core::{LintContext, Rule};
use rubric_rules::layout::case_indentation::CaseIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/case_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/case_indentation/corrected.rb");

#[test]
fn detects_bad_case_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CaseIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/CaseIndentation"));
}

#[test]
fn no_violation_for_correct_case_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = CaseIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
