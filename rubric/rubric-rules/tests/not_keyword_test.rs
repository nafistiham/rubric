use rubric_core::{LintContext, Rule};
use rubric_rules::style::not_keyword::NotKeyword;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/not_keyword/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/not_keyword/corrected.rb");

#[test]
fn detects_not_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NotKeyword.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/Not"));
}

#[test]
fn no_violation_for_bang_operator() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_for_where_not_method_call() {
    let src = "scope :remote, -> { where.not(domain: nil) }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for where.not(), got: {:?}", diags);
}
