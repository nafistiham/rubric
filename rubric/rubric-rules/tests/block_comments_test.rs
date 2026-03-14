use rubric_core::{LintContext, Rule};
use rubric_rules::style::block_comments::BlockComments;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/block_comments/offending.rb");
const PASSING: &str = include_str!("fixtures/style/block_comments/passing.rb");

#[test]
fn detects_block_comment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockComments.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/BlockComments"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = BlockComments.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_correct_message() {
    let src = "=begin\nsome comment\n=end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockComments.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(
        diags[0].message.contains("block comments"),
        "message should mention block comments"
    );
}

#[test]
fn does_not_flag_indented_begin() {
    // =begin must be at column 0 to be a block comment in Ruby
    let src = "  =begin\nsome text\n  =end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockComments.check_source(&ctx);
    assert!(diags.is_empty(), "indented =begin should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_hash_comment() {
    let src = "# This is a line comment\ndef foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockComments.check_source(&ctx);
    assert!(diags.is_empty(), "hash comments should not be flagged, got: {:?}", diags);
}
