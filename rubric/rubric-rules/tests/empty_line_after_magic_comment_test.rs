use rubric_core::{LintContext, Rule};
use rubric_rules::layout::empty_line_after_magic_comment::EmptyLineAfterMagicComment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/empty_line_after_magic_comment/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/layout/empty_line_after_magic_comment/clean.rb");

#[test]
fn detects_missing_blank_line_after_magic_comment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EmptyLineAfterMagicComment.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation");
    assert_eq!(diags[0].rule, "Layout/EmptyLineAfterMagicComment");
}

#[test]
fn no_violation_when_blank_line_present() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = EmptyLineAfterMagicComment.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations");
}

#[test]
fn no_violation_when_no_magic_comments() {
    let source = "class Foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = EmptyLineAfterMagicComment.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations without magic comments");
}

#[test]
fn handles_multiple_magic_comments() {
    let source = "# frozen_string_literal: true\n# encoding: utf-8\nclass Foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = EmptyLineAfterMagicComment.check_source(&ctx);
    assert_eq!(diags.len(), 1, "expected 1 violation after the last magic comment");
}

#[test]
fn no_violation_when_magic_comment_is_last_line() {
    let source = "# frozen_string_literal: true\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = EmptyLineAfterMagicComment.check_source(&ctx);
    assert_eq!(diags.len(), 0, "expected no violations when magic comment is last line");
}
