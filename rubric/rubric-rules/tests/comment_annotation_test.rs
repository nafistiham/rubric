use rubric_core::{LintContext, Rule};
use rubric_rules::style::comment_annotation::CommentAnnotation;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/comment_annotation/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/comment_annotation/clean.rb");

#[test]
fn detects_malformed_annotations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/CommentAnnotation"));
}

#[test]
fn no_violation_for_correct_annotations() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_todo_without_colon() {
    let src = "# TODO fix this\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(!diags.is_empty(), "TODO without colon should be flagged");
    assert!(diags[0].message.contains("TODO"));
}

#[test]
fn flags_todo_alone() {
    let src = "# TODO\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(!diags.is_empty(), "bare TODO should be flagged");
}

#[test]
fn flags_todo_colon_no_space() {
    let src = "# TODO:no space\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(!diags.is_empty(), "TODO:no_space should be flagged");
}

#[test]
fn flags_optimize_colon_only() {
    // "# OPTIMIZE: " with only whitespace after colon+space is still missing description
    let src = "# OPTIMIZE: \n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(!diags.is_empty(), "OPTIMIZE with no description should be flagged");
}

#[test]
fn does_not_flag_annotation_in_code() {
    // keyword inside a string should not be flagged — no leading `#`
    let src = "msg = \"TODO fix this\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(diags.is_empty(), "TODO inside string should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_correct_fixme() {
    let src = "# FIXME: something needs fixing\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = CommentAnnotation.check_source(&ctx);
    assert!(diags.is_empty(), "correct FIXME annotation should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_all_annotation_keywords() {
    for kw in &["FIXME", "OPTIMIZE", "HACK", "REVIEW", "NOTE", "XXX"] {
        let src = format!("# {} bad format\n", kw);
        let ctx = LintContext::new(Path::new("test.rb"), &src);
        let diags = CommentAnnotation.check_source(&ctx);
        assert!(!diags.is_empty(), "{} without colon should be flagged", kw);
        assert!(diags[0].message.contains(kw), "message should mention keyword {}", kw);
    }
}
