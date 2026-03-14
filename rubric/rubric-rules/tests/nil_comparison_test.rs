use rubric_core::{LintContext, Rule};
use rubric_rules::style::nil_comparison::NilComparison;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/nil_comparison/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/nil_comparison/clean.rb");

#[test]
fn detects_eq_nil() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NilComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/NilComparison"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = NilComparison.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment() {
    let src = "# x == nil is bad\nfoo.nil?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NilComparison.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = r#"msg = "use == nil carefully"
other = 'x != nil here'
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NilComparison.check_source(&ctx);
    assert!(diags.is_empty(), "string contents should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_eq_nil_with_correct_message() {
    let src = "if x == nil\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NilComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(diags[0].message.contains("nil?"), "message should mention nil?");
}

#[test]
fn flags_neq_nil() {
    let src = "if x != nil\n  1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NilComparison.check_source(&ctx);
    assert!(!diags.is_empty(), "!= nil should be flagged");
}

#[test]
fn skips_heredoc_body() {
    let src = "sql = <<~SQL\n  WHERE x == nil\nSQL\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NilComparison.check_source(&ctx);
    assert!(diags.is_empty(), "heredoc body should not be flagged, got: {:?}", diags);
}
