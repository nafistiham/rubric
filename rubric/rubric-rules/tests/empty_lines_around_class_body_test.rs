use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines_around_class_body::EmptyLinesAroundClassBody;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_lines_around_class_body/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/empty_lines_around_class_body/corrected.rb");

#[test]
fn detects_empty_lines_around_class_body() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLinesAroundClassBody.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLinesAroundClassBody"));
}

#[test]
fn no_violation_for_clean_class_body() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLinesAroundClassBody.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_single_line_nested_class() {
    // `class Error < StandardError; end` on one line is not a multi-line body opener.
    // An empty line after it must not trigger "Extra empty line after class body start".
    let src = "class Outer\n  class Error < StandardError; end\n\n  def foo\n    1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundClassBody.check_source(&ctx);
    assert!(diags.is_empty(), "single-line nested class should not be flagged, got: {:?}", diags);
}
