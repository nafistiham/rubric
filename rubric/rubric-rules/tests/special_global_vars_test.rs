use rubric_core::{LintContext, Rule};
use rubric_rules::style::special_global_vars::SpecialGlobalVars;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/special_global_vars/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/special_global_vars/clean.rb");

#[test]
fn detects_perl_style_globals() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/SpecialGlobalVars"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_dollar_zero() {
    let src = "puts $0\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(!diags.is_empty(), "$0 should be flagged");
    assert_eq!(diags[0].rule, "Style/SpecialGlobalVars");
    assert!(
        diags[0].message.contains("$PROGRAM_NAME"),
        "message should mention $PROGRAM_NAME, got: {}",
        diags[0].message
    );
}

#[test]
fn flags_dollar_colon_load_path() {
    let src = "$: << '/usr/local/lib'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(!diags.is_empty(), "$: should be flagged");
    assert!(
        diags[0].message.contains("$LOAD_PATH"),
        "message should mention $LOAD_PATH, got: {}",
        diags[0].message
    );
}

#[test]
fn flags_dollar_bang_error_info() {
    let src = "puts $!\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(!diags.is_empty(), "$! should be flagged");
    assert!(
        diags[0].message.contains("$ERROR_INFO"),
        "message should mention $ERROR_INFO, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_comment() {
    let src = "# $0 in a comment is fine\n# $! too\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_english_names() {
    let src = "puts $PROGRAM_NAME\n$LOAD_PATH << '/lib'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert!(diags.is_empty(), "English names should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_multiple_globals_on_separate_lines() {
    let src = "puts $0\nputs $!\nputs $@\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpecialGlobalVars.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations, got: {:?}", diags);
}
