use rubric_core::{LintContext, Rule};
use rubric_rules::style::select_by_regexp::SelectByRegexp;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/select_by_regexp/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/select_by_regexp/clean.rb");

#[test]
fn detects_select_with_regexp() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SelectByRegexp"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_select_with_match_operator() {
    let src = "items.select { |item| item =~ /foo/ }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(!diags.is_empty(), "select with =~ should be flagged");
    assert!(diags[0].message.contains("grep"));
}

#[test]
fn flags_reject_with_match_operator() {
    let src = "items.reject { |item| item =~ /bar/ }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(!diags.is_empty(), "reject with =~ should be flagged");
    assert!(diags[0].message.contains("grep_v"));
}

#[test]
fn flags_select_with_match_method() {
    let src = "names.select { |n| n.match?(/^foo/) }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(!diags.is_empty(), "select with match? should be flagged");
}

#[test]
fn does_not_flag_select_without_regexp() {
    let src = "names.select { |n| n.start_with?(\"foo\") }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(diags.is_empty(), "select without regexp should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_grep_directly() {
    let src = "items.grep(/foo/)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SelectByRegexp.check_source(&ctx);
    assert!(diags.is_empty(), "grep should not be flagged, got: {:?}", diags);
}

#[test]
fn counts_all_offending_lines() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SelectByRegexp.check_source(&ctx);
    assert_eq!(diags.len(), 3, "should flag all 3 offending lines, got: {:?}", diags);
}
