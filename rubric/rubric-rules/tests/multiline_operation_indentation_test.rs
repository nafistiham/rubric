use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_operation_indentation::MultilineOperationIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/multiline_operation_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/multiline_operation_indentation/corrected.rb");

#[test]
fn detects_bad_multiline_operation_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineOperationIndentation"));
}

#[test]
fn no_violation_for_correct_multiline_operation_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MultilineOperationIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
