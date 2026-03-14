use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/lint/safe_navigation_with_empty.rs"]
mod safe_navigation_with_empty;
use safe_navigation_with_empty::SafeNavigationWithEmpty;

const OFFENDING: &str = include_str!("fixtures/lint/safe_navigation_with_empty/offending.rb");
const PASSING: &str = include_str!("fixtures/lint/safe_navigation_with_empty/passing.rb");

#[test]
fn detects_safe_navigation_with_empty() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/SafeNavigationWithEmpty"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_with_correct_message() {
    let src = "return if foo&.empty?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(
        diags[0].message.contains("safe navigation"),
        "message should mention safe navigation, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_regular_empty() {
    let src = "return if foo.empty?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), ".empty? without safe nav should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_blank() {
    let src = "return if foo.blank?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), ".blank? should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_nil_or_empty() {
    let src = "return if foo.nil? || foo.empty?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), "nil? || empty? should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment() {
    let src = "# foo&.empty? is an anti-pattern\nreturn if foo.nil? || foo.empty?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = r#"msg = "avoid foo&.empty? usage"
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), "string contents should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_heredoc_body() {
    let src = "doc = <<~TEXT\n  foo&.empty? is bad\nTEXT\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert!(diags.is_empty(), "heredoc body should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_multiple_occurrences() {
    let src = "return if foo&.empty?\nraise if bar&.empty?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SafeNavigationWithEmpty.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected two violations, got: {:?}", diags);
}
