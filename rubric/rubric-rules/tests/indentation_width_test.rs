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

// ── Inline if continuation alignment must NOT fire ─────────────────────────
// `x = if cond\n       value\n     end` — the `value` and `end` are aligned
// to the `if` keyword, resulting in odd-numbered indents.
#[test]
fn no_false_positive_for_inline_if_continuation() {
    let src = concat!(
        "def email(name: nil, domain: nil)\n",
        "  local_part = if domain\n",
        "                 foo(name: name, domain: domain)\n",
        "               else\n",
        "                 foo(name: name)\n",
        "               end\n",
        "  local_part\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "inline if alignment should not be flagged: {:?}", diags);
}

#[test]
fn no_false_positive_for_inline_unless_continuation() {
    let src = concat!(
        "def foo\n",
        "  result = unless condition\n",
        "             value_a\n",
        "           else\n",
        "             value_b\n",
        "           end\n",
        "  result\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "inline unless alignment should not be flagged: {:?}", diags);
}
