use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_lines_around_access_modifier::EmptyLinesAroundAccessModifier;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_lines_around_access_modifier/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/layout/empty_lines_around_access_modifier/clean.rb");

#[test]
fn detects_missing_blank_lines_around_access_modifier() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLinesAroundAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EmptyLinesAroundAccessModifier"));
}

#[test]
fn no_violation_for_properly_spaced_access_modifier() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = EmptyLinesAroundAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_when_modifier_at_top_of_class() {
    let src = "class Foo\n  private\n\n  def bar; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "modifier at top of class should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_when_modifier_at_bottom_of_class() {
    let src = "class Foo\n  def bar; end\n\n  private\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "modifier at bottom of class should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_missing_blank_line_before_modifier() {
    let src = "class Foo\n  def foo; end\n  private\n\n  def bar; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for missing blank before private");
}

#[test]
fn detects_missing_blank_line_after_modifier() {
    let src = "class Foo\n  def foo; end\n\n  private\n  def bar; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EmptyLinesAroundAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for missing blank after private");
}
