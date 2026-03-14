use rubric_core::{LintContext, Rule};
use rubric_rules::style::when_then::WhenThen;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/when_then/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/when_then/clean.rb");

#[test]
fn detects_when_then_on_multiline() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = WhenThen.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/WhenThen"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = WhenThen.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_when_then_at_end_of_line() {
    let src = "case x\nwhen 1 then\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhenThen.check_source(&ctx);
    assert!(!diags.is_empty(), "when...then at end of line should be flagged");
}

#[test]
fn does_not_flag_inline_when_then() {
    let src = "case x\nwhen 1 then foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhenThen.check_source(&ctx);
    assert!(diags.is_empty(), "inline when...then should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_when_without_then() {
    let src = "case x\nwhen 1\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhenThen.check_source(&ctx);
    assert!(diags.is_empty(), "when without then should not be flagged, got: {:?}", diags);
}

#[test]
fn message_mentions_then() {
    let src = "case x\nwhen 1 then\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = WhenThen.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("then"),
        "message should mention 'then', got: {}",
        diags[0].message
    );
}

#[test]
fn counts_all_offending_when_clauses() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = WhenThen.check_source(&ctx);
    assert_eq!(diags.len(), 2, "should flag both offending when...then lines, got: {:?}", diags);
}
