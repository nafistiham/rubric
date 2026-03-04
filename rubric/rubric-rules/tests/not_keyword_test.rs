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

// ── Heredoc bodies must not be flagged ────────────────────────────────────────
#[test]
fn no_false_positive_for_not_in_heredoc_squiggly() {
    let src = "abort <<~ERROR\n  The value is not set.\n  Do not change it.\nERROR\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "not inside <<~HEREDOC should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_not_in_heredoc_dash() {
    let src = "msg = <<-LONG_DESC\n  Does not apply here.\nLONG_DESC\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "not inside <<-HEREDOC should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_not_in_heredoc_bare() {
    let src = "puts <<MSG\nnot a keyword here\nMSG\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "not inside <<HEREDOC should not be flagged: {:?}", diags);
}

// Ensure real `not` usage on the opener line (before heredoc content) is still flagged
#[test]
fn detects_not_on_heredoc_opener_line() {
    // `not` appears on the opener line itself (before the <<), not in the body
    let src = "x = not foo\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(!diags.is_empty(), "not on a normal line must still be flagged");
}

// ── `not` inside a regex literal must not be flagged ─────────────────────────
#[test]
fn no_false_positive_for_not_in_regex_literal() {
    let src = "expect(msg).to match(/action is not allowed/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "not inside /regex/ should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_not_in_regex_raise_error() {
    let src = ".to raise_error(Thor::Error, /DOMAIN parameter not supported/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NotKeyword.check_source(&ctx);
    assert!(diags.is_empty(), "not inside /regex/ in raise_error should not be flagged: {:?}", diags);
}
