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

#[test]
fn no_false_positive_for_shovel_if_continuation() {
    let src = "arr << if condition\n           val1\n         else\n           val2\n         end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for << if continuation, got: {:?}", diags);
}

// ── Array elements aligned to opening `[` must NOT fire ──────────────────────
// `sample([` on one line, elements at column 19 (alignment) — odd but valid.
#[test]
fn no_false_positive_for_array_alignment_indentation() {
    let src = concat!(
        "        sample([\n",
        "                 :first_element,\n",
        "                 :second_element,\n",
        "               ])\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "array alignment indentation should not be flagged: {:?}", diags);
}

// ── Hash/method-call alignment after opening `(` must NOT fire ───────────────
#[test]
fn no_false_positive_for_hash_alignment_after_open_paren() {
    let src = concat!(
        "  reblog: [\n",
        "    :application,\n",
        "    :media_attachments,\n",
        "  ]\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(diags.is_empty(), "content after [ on its own line should not be flagged: {:?}", diags);
}

// ── Continuation line inside %i[] literal aligns to bracket — must NOT fire ──
// `CREDIT_CARD_TYPES = %i[visa mastercard\n                       diners_club]`
// Line 2 has 23-space indent (alignment to `[`) — odd but valid.
#[test]
fn no_false_positive_for_percent_literal_bracket_continuation() {
    let src = concat!(
        "CREDIT_CARD_TYPES = %i[visa mastercard discover american_express\n",
        "                       diners_club jcb switch solo].freeze\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "continuation line inside bracket expression should not be flagged: {:?}",
        diags
    );
}

// ── Multiple lines inside open bracket — all should be skipped ───────────────
#[test]
fn no_false_positive_for_multiline_array_literal_continuation() {
    let src = concat!(
        "THINGS = [\n",
        "  :foo,\n",
        "  :bar,\n",
        "  :baz\n",
        "]\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "lines inside open array bracket should not be flagged: {:?}",
        diags
    );
}
