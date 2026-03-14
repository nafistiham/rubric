use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/style/case_equality.rs"]
mod case_equality;
use case_equality::CaseEquality;

const OFFENDING: &str = include_str!("fixtures/style/case_equality/offending.rb");
const PASSING: &str = include_str!("fixtures/style/case_equality/passing.rb");

#[test]
fn detects_case_equality_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CaseEquality.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/CaseEquality"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = CaseEquality.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_with_correct_message() {
    let src = "if String === obj\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseEquality.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(
        diags[0].message.contains("==="),
        "message should mention ===, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_comment() {
    let src = "# String === obj is bad\nobj.is_a?(String)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseEquality.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = r#"msg = "String === obj"
other = 'Integer === val'
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseEquality.check_source(&ctx);
    assert!(diags.is_empty(), "string contents should not be flagged, got: {:?}", diags);
}

#[test]
fn skips_heredoc_body() {
    let src = "doc = <<~TEXT\n  String === obj\nTEXT\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseEquality.check_source(&ctx);
    assert!(diags.is_empty(), "heredoc body should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_inline_usage() {
    let src = "result = Integer === value\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CaseEquality.check_source(&ctx);
    assert!(!diags.is_empty(), "inline === should be flagged");
}
