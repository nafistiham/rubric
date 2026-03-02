use rubric_core::{LintContext, Rule};
use rubric_rules::layout::indentation_width::IndentationWidth;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/indentation_width/offending.rb");
const CORRECTED: &str = include_str!("fixtures/layout/indentation_width/corrected.rb");

#[test]
fn detects_wrong_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/IndentationWidth"));
}

#[test]
fn no_violation_on_correct_indentation() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty());
}

// ── Aligned continuation args after comma must NOT fire ───────────────────
// `delegate :foo,\n         :bar` — `:bar` has 11-space indent for alignment.
// This is a continuation line, not a new block scope.
#[test]
fn no_false_positive_for_aligned_continuation_after_comma() {
    let src = "delegate :email,\n         :email_domain,\n         to: :user\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "aligned continuation args should not be flagged: {:?}", diags);
}

// ── Method call with aligned hash args must NOT fire ──────────────────────
#[test]
fn no_false_positive_for_aligned_method_args() {
    let src = "foo(bar:   1,\n    baz:   2,\n    quux:  3)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "aligned method args should not be flagged: {:?}", diags);
}

// ── True positive: 3-space block indentation still fires ──────────────────
#[test]
fn still_detects_odd_block_indentation() {
    let src = "def foo\n   puts 'bad'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(!diags.is_empty(), "3-space indent should still be flagged");
}
