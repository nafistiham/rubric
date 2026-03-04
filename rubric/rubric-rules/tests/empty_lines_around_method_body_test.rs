use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines_around_method_body::EmptyLinesAroundMethodBody;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_lines_around_method_body/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/empty_lines_around_method_body/corrected.rb");

#[test]
fn detects_empty_lines_around_method_body() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLinesAroundMethodBody"));
}

#[test]
fn no_violation_for_clean_method_body() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_single_line_method() {
    // `def show; end` is a one-liner — the blank line after it is not "inside" the body.
    let src = "class Foo\n  def show; end\n\n  def update\n    do_something\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(diags.is_empty(), "single-line def should not be flagged, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_single_line_method_with_body() {
    // `def foo; 42; end` — still a one-liner, blank line after must not be flagged.
    let src = "class Foo\n  def foo; 42; end\n\n  def bar\n    1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(diags.is_empty(), "single-line method with body should not be flagged, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_ruby3_endless_method() {
    // Ruby 3 endless method `def foo = expr` has no multi-line body.
    // The blank line after it is not "inside" the body.
    let src = "class Foo\n  def self.name = 'Foo'\n\n  def to_s = name\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundMethodBody.check_source(&ctx);
    assert!(diags.is_empty(), "endless method should not be flagged, got: {:?}", diags);
}
