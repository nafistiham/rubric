use rubric_core::{LintContext, Rule};
use rubric_rules::style::file_null::FileNull;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/file_null/offending.rb");
const PASSING: &str = include_str!("fixtures/style/file_null/passing.rb");

#[test]
fn detects_dev_null_double_quote() {
    let src = "File.open(\"/dev/null\", \"w\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FileNull.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for \"/dev/null\", got none");
    assert!(diags.iter().all(|d| d.rule == "Style/FileNull"));
}

#[test]
fn detects_dev_null_single_quote() {
    let src = "redirect_output('/dev/null')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FileNull.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for '/dev/null', got none");
    assert!(diags.iter().all(|d| d.rule == "Style/FileNull"));
}

#[test]
fn no_violation_for_file_null_constant() {
    let src = "File.open(File::NULL, \"w\")\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FileNull.check_source(&ctx);
    assert!(diags.is_empty(), "File::NULL should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_comment_line() {
    let src = "# use /dev/null to discard output\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FileNull.check_source(&ctx);
    assert!(diags.is_empty(), "comment line should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_violation_in_offending_fixture() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FileNull.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/FileNull"));
}

#[test]
fn no_violation_in_passing_fixture() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = FileNull.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations in passing.rb, got: {:?}", diags);
}
