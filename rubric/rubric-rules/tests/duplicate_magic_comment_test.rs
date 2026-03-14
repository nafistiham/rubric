use rubric_core::{LintContext, Rule};
use rubric_rules::lint::duplicate_magic_comment::DuplicateMagicComment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/duplicate_magic_comment/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/lint/duplicate_magic_comment/clean.rb");

#[test]
fn detects_duplicate_magic_comment() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/DuplicateMagicComment"));
}

#[test]
fn no_violation_for_unique_magic_comments() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_duplicate_frozen_string_literal() {
    let src = "# frozen_string_literal: true\n# frozen_string_literal: false\n\nclass Foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for duplicate frozen_string_literal");
    assert_eq!(diags.len(), 1, "expected exactly 1 violation");
}

#[test]
fn flags_duplicate_encoding_comment() {
    let src = "# encoding: utf-8\n# encoding: ascii\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for duplicate encoding comment");
}

#[test]
fn no_violation_for_file_with_no_magic_comments() {
    let src = "class Foo\n  def bar\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn duplicate_detection_is_case_insensitive() {
    let src = "# Frozen_String_Literal: true\n# frozen_string_literal: true\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(!diags.is_empty(), "case-insensitive duplicate should be flagged");
}

#[test]
fn violation_message_mentions_duplicate() {
    let src = "# frozen_string_literal: true\n# frozen_string_literal: true\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DuplicateMagicComment.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("Duplicate"),
        "message should mention Duplicate, got: {}",
        diags[0].message
    );
}
