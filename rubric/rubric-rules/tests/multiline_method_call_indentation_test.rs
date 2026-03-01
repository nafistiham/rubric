use rubric_core::{LintContext, Rule};
use rubric_rules::layout::multiline_method_call_indentation::MultilineMethodCallIndentation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/multiline_method_call_indentation/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/multiline_method_call_indentation/corrected.rb");

#[test]
fn detects_trailing_dot_in_chained_call() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for trailing dots, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/MultilineMethodCallIndentation"));
}

#[test]
fn no_violation_with_leading_dots() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MultilineMethodCallIndentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}
