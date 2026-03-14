use rubric_core::{LintContext, Rule};
use rubric_rules::style::array_intersect::ArrayIntersect;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/array_intersect/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/array_intersect/clean.rb");

#[test]
fn detects_array_intersection_any() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ArrayIntersect.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ArrayIntersect"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ArrayIntersect.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_inline_pattern() {
    let src = "result = (a & b).any?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayIntersect.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for (a & b).any?");
    assert_eq!(diags[0].rule, "Style/ArrayIntersect");
    assert!(diags[0].message.contains("intersect?"), "message should mention intersect?");
}

#[test]
fn does_not_flag_intersect_method() {
    let src = "a.intersect?(b)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayIntersect.check_source(&ctx);
    assert!(diags.is_empty(), "intersect? usage should not be flagged");
}

#[test]
fn does_not_flag_comment() {
    let src = "# (a & b).any? is bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayIntersect.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_if_condition_pattern() {
    let src = "if (users & admins).any?\n  true\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ArrayIntersect.check_source(&ctx);
    assert!(!diags.is_empty(), "if-condition pattern should be flagged");
}
