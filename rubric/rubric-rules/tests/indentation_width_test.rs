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

// ── Heredoc body lines must NOT fire ─────────────────────────────────────────
// Lines inside a <<-TERM / <<~TERM heredoc are raw string content, not Ruby
// code, so their indentation is irrelevant to the IndentationWidth rule.
#[test]
fn no_false_positive_for_heredoc_body_lines() {
    let src = concat!(
        "create_view \"account_summaries\", sql_definition: <<-SQL\n",
        "   SELECT accounts.id AS account_id,\n",    // 3 spaces — odd
        "     FROM accounts\n",                       // 5 spaces — odd
        "SQL\n",
        "add_index \"account_summaries\", [\"account_id\"]\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "heredoc body lines should not be flagged: {:?}",
        diags
    );
}

// ── Squiggly heredoc (<<~) body lines must NOT fire ──────────────────────────
#[test]
fn no_false_positive_for_squiggly_heredoc_body_lines() {
    let src = concat!(
        "message = <<~MSG\n",
        "   Hello world\n",   // 3 spaces — odd, but inside heredoc
        "MSG\n",
        "puts message\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "squiggly heredoc body lines should not be flagged: {:?}",
        diags
    );
}

// ── ||= if continuation alignment must NOT fire ──────────────────────────────
// `@var ||= if condition\n             value\n           end`
// The body is aligned to the `if` keyword, producing odd-number indents.
#[test]
fn no_false_positive_for_memoized_inline_if_continuation() {
    let src = concat!(
        "def reports\n",
        "  @reports ||= if type == 'none'\n",
        "                 [report]\n",
        "               else\n",
        "                 target_account.targeted_reports\n",
        "               end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "||= if alignment should not be flagged: {:?}",
        diags
    );
}

// ── Backslash line-continuation must NOT fire ─────────────────────────────────
// `raise ArgumentError, "long message" \
//                       unless condition`
// The continuation line is aligned to the string start — odd indent, valid.
#[test]
fn no_false_positive_for_backslash_continuation() {
    let src = concat!(
        "def foo\n",
        "  raise ArgumentError, \"bad argument\" \\\n",
        "                       unless valid?\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "backslash continuation line should not be flagged: {:?}",
        diags
    );
}

// ── Boolean &&/|| continuation must NOT fire ──────────────────────────────────
// Multi-line boolean expressions where each operand is aligned to the first:
//   if record.respond_to?(:foo) &&
//      record.bar?
// The second line has 5-space indent — odd but conventional.
#[test]
fn no_false_positive_for_boolean_and_continuation() {
    let src = concat!(
        "Warden::Manager.after_set_user do |record, warden, options|\n",
        "  if record.respond_to?(:remember_me) && options[:store] != false &&\n",
        "     record.remember_me && warden.authenticated?(scope)\n",
        "    do_something\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "&& boolean continuation should not be flagged: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_boolean_or_continuation() {
    let src = concat!(
        "def foo\n",
        "  result = a_long_condition? ||\n",
        "           another_condition?\n",
        "  result\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "|| boolean continuation should not be flagged: {:?}",
        diags
    );
}

// ── Method chain with leading dot must NOT fire ───────────────────────────────
// `Rails\n  .application\n  .config` — the `.method` lines are aligned to the
// receiver. Also covers `"string".chars\n                .insert(3, ' ')`.
#[test]
fn no_false_positive_for_method_chain_dot() {
    let src = concat!(
        "Rails\n",
        "  .application\n",
        "  .config\n",
        "  .session_store :cookie_store,\n",
        "                 key: '_session',\n",
        "                 same_site: :lax\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "method chain dot lines should not be flagged: {:?}",
        diags
    );
}

// ── Arithmetic/string operator at end of line continuation must NOT fire ─────
// `entities = extract_urls(...) +\n               extract_hashtags(...)` —
// the continuation is aligned to the first operand (15-space indent = odd).
#[test]
fn no_false_positive_for_arithmetic_operator_continuation() {
    let src = concat!(
        "def extract_entities(text)\n",
        "  entities = extract_urls_with_indices(text) +\n",
        "               extract_hashtags_with_indices(text) +\n",
        "               extract_mentions_with_indices(text)\n",
        "  entities\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "operator continuation lines should not be flagged: {:?}",
        diags
    );
}

// ── Comment-masked trailing comma must NOT fire ───────────────────────────────
// When the previous line has `key: value, # comment`, the trailing comma is
// hidden behind the comment. The next aligned arg line should still be skipped.
#[test]
fn no_false_positive_for_comment_masked_trailing_comma() {
    let src = concat!(
        ".session_store :cookie_store,\n",
        "               key: '_session',\n",
        "               secure: false, # All cookies use force_ssl\n",
        "               same_site: :lax\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IndentationWidth.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "comment-masked trailing comma should not defeat comma-continuation check: {:?}",
        diags
    );
}
