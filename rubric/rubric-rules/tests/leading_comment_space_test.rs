use rubric_core::{LintContext, Rule};
use rubric_rules::layout::leading_comment_space::LeadingCommentSpace;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/leading_comment_space/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/leading_comment_space/corrected.rb");

#[test]
fn detects_comment_without_space() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/LeadingCommentSpace"));
}

#[test]
fn no_violation_with_space_after_hash() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_for_double_hash_yard_comments() {
    let src = "## @param name [String] the name\n## @return [void]\ndef foo(name)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for ## YARD comments, got: {:?}", diags);
}

// ── Heredoc contents must not be treated as comments ─────────────────────────
// Lines inside a heredoc (<<~MSG, <<-RUBY, %{...}) may begin with `#` as part
// of Ruby string interpolation `#{expr}`. They are not comments and must not
// be flagged by LeadingCommentSpace.
#[test]
fn no_violation_for_interpolation_in_heredoc() {
    // Simulates: raise <<~MSG \n  #build_enumerator must return ...\nMSG
    // The `#build_enumerator` line is inside a heredoc — not a comment.
    let src = "raise ArgumentError, <<~MSG\n  #build_enumerator must return an Enumerator.\nMSG\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "heredoc body starting with `#` should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_interpolation_in_percent_literal() {
    // Simulates the sidekiq ASCII-art banner: %{\n  #{w}   text\n}
    // The `#{w}` line is inside a %{...} percent literal string — not a comment.
    let src = "banner = %{\n  #{w}   some text\n  #{b}   other text\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "percent-literal body lines starting with `#{{` should not be flagged, got: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_interpolation_in_double_quoted_heredoc() {
    // <<-RUBY heredoc with interpolation on first non-space character of a line.
    let src = "class_eval <<-RUBY, __FILE__, __LINE__\n  def _erb_#{name}\n    #{src}\n  end\nRUBY\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = LeadingCommentSpace.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "interpolation lines inside <<-RUBY heredoc should not be flagged, got: {:?}",
        diags
    );
}
