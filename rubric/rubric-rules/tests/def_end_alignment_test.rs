use rubric_core::{LintContext, Rule};
use rubric_rules::layout::def_end_alignment::DefEndAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/def_end_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/DefEndAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def bar\n  'bar'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_do_block_inside_def() {
    let src = concat!(
        "def foo\n",
        "  case x\n",
        "  when :a\n",
        "    [1,2].each do |i|\n",
        "      puts i\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for do-block inside def with case, got: {:?}", diags);
}

#[test]
fn still_detects_misaligned_def_end() {
    let src = "def foo\n  1\n    end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned def end, got none");
}

#[test]
fn no_violation_for_inline_if_inside_def() {
    // `end` at aligned position closes the inline `if`, not the `def`
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
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for inline if inside def, got: {:?}", diags);
}

#[test]
fn no_violation_for_multiple_inline_ifs_inside_def() {
    let src = concat!(
        "def foo\n",
        "  a = if x\n",
        "        1\n",
        "      else\n",
        "        2\n",
        "      end\n",
        "  b = unless y\n",
        "        3\n",
        "      else\n",
        "        4\n",
        "      end\n",
        "  a + b\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for multiple inline ifs, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_shovel_if_inside_def() {
    let src = "def foo\n  arr << if cond\n             val1\n           else\n             val2\n           end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for << if inside def, got: {:?}", diags);
}

// ── False positive: endless method `def foo = expr` (no `end` needed) ────────
#[test]
fn no_false_positive_for_endless_method() {
    let src = concat!(
        "module Foo\n",
        "  class Bar\n",
        "    def name = \"bar\"\n",
        "    def count = 42\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "endless methods falsely flagged: {:?}", diags);
}

// ── False positive: endless method with params `def foo(x) = expr` ────────────
#[test]
fn no_false_positive_for_endless_method_with_params() {
    let src = concat!(
        "module Router\n",
        "  def head(path, &) = route(:head, path, &)\n",
        "  def get(path, &) = route(:get, path, &)\n",
        "  def post(path, &) = route(:post, path, &)\n",
        "  def route(*methods, path, &block)\n",
        "    methods.each { |m| routes[m] << block }\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "endless methods with params falsely flagged: {:?}", diags);
}

// ── False positive: one-liner class `class Foo; end` ─────────────────────────
#[test]
fn no_false_positive_for_one_liner_class() {
    let src = concat!(
        "module Sidekiq\n",
        "  class JobRetry\n",
        "    class Handled < ::RuntimeError; end\n",
        "    class Skip < Handled; end\n",
        "    def initialize(capsule)\n",
        "      @capsule = capsule\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "one-liner class falsely flagged: {:?}", diags);
}

// ── False positive: `private def` should align like a normal `def` ───────────
#[test]
fn no_false_positive_for_private_def() {
    let src = concat!(
        "class Config\n",
        "  def public_method\n",
        "    1\n",
        "  end\n",
        "\n",
        "  private def parameter_size(handler)\n",
        "    handler.parameters.size\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "private def falsely flagged: {:?}", diags);
}

// ── False positive: `protected def` should align like a normal `def` ─────────
#[test]
fn no_false_positive_for_protected_def() {
    let src = concat!(
        "class Foo\n",
        "  protected def bar\n",
        "    42\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "protected def falsely flagged: {:?}", diags);
}

// ── False positive: heredoc body with def/end keywords inside ─────────────────
#[test]
fn no_false_positive_for_heredoc_body_with_def_end() {
    let src = concat!(
        "class Foo\n",
        "  def bar\n",
        "    class_eval <<-METHODS\n",
        "      def baz\n",
        "        42\n",
        "      end\n",
        "    METHODS\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for heredoc body with def/end, got: {:?}", diags);
}

// ── False positive: squiggly heredoc `<<~RUBY` with def/end inside ───────────
#[test]
fn no_false_positive_for_squiggly_heredoc_body_with_def_end() {
    let src = concat!(
        "module Devise\n",
        "  def self.setup\n",
        "    class_eval <<~METHODS\n",
        "      def authenticate!\n",
        "        warden.authenticate!\n",
        "      end\n",
        "    METHODS\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for squiggly heredoc body with def/end, got: {:?}", diags);
}

// ── False positive: inline `if` with nested inline `begin` inside def ─────────
#[test]
fn no_false_positive_for_inline_if_with_nested_inline_begin() {
    let src = concat!(
        "def fetch\n",
        "  result = if entry\n",
        "    job = begin\n",
        "      load(entry)\n",
        "    rescue\n",
        "      {}\n",
        "    end\n",
        "    compute(job)\n",
        "  else\n",
        "    0.0\n",
        "  end\n",
        "  result\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "nested inline begin in inline if falsely flagged: {:?}", diags);
}

// ── False positive: `end,` (do-block end inside method argument list) ─────────
// Mirrors mastodon/app/helpers/formatting_helper.rb pattern
#[test]
fn no_false_positive_for_do_block_end_comma_in_method_args() {
    // The `end,` closes the do-block passed as the first arg to safe_join.
    // The def's own `end` is at indent 2, matching the def at indent 2.
    let src = concat!(
        "class FormattingHelper\n",
        "  def poll_option_tags(status)\n",
        "    safe_join(\n",
        "      status.options.map do |option|\n",
        "        option\n",
        "      end,\n",       // end, — do-block end inside argument list
        "      tag.br\n",
        "    )\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end, inside method args falsely flagged: {:?}", diags);
}

// ── False positive: `end)` (do-block end closes a method call group) ──────────
#[test]
fn no_false_positive_for_do_block_end_paren_in_method_args() {
    // `end)` closes the do-block AND the enclosing method call parens.
    let src = concat!(
        "class Foo\n",
        "  def bar\n",
        "    result = arr.map(items.each do |x|\n",
        "      x * 2\n",
        "    end)\n",       // end) — do-block end that also closes parens
        "    result\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end) inside method args falsely flagged: {:?}", diags);
}

// ── False positive: multiple `end,` inside one def — stack must stay correct ──
#[test]
fn no_false_positive_for_multiple_do_block_end_comma() {
    let src = concat!(
        "module M\n",
        "  def foo\n",
        "    safe_join(\n",
        "      a.map do |x|\n",
        "        x\n",
        "      end,\n",
        "      b.map do |y|\n",
        "        y\n",
        "      end,\n",
        "      tag.br\n",
        "    )\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "multiple end, inside method args falsely flagged: {:?}", diags);
}

// ── False positive: `end:` (hash symbol key) must not be treated as `end` ─────
#[test]
fn no_false_positive_for_hash_end_symbol() {
    // `end:` at the start of a trimmed line (multi-line hash) is a hash symbol
    // key, not an `end` token. Must not pop the def frame.
    let src = concat!(
        "def foo\n",
        "  opts = {\n",
        "    end: 5,\n",
        "    start: 1\n",
        "  }\n",
        "  opts\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end: hash symbol falsely flagged: {:?}", diags);
}

// ── False positive: inline `if` after arithmetic/logical operator ──────────────
#[test]
fn no_false_positive_for_inline_if_after_operator() {
    // `acc + if index.even?` opens a conditional block; its `end` must not pop
    // the def frame prematurely.
    let src = concat!(
        "def sum_alternating(arr)\n",
        "  arr.each_with_index.inject(0) do |acc, (val, index)|\n",
        "    acc + if index.even?\n",
        "             val\n",
        "           else\n",
        "             -val\n",
        "           end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline if after + operator falsely flagged: {:?}", diags);
}

// ── False positive: backslash continuation line starting with `unless` ─────────
// e.g. `raise Err, 'msg' \` / `  unless condition` — the `unless` is a
// modifier on the continuation, not a new block opener.
#[test]
fn no_false_positive_for_unless_on_continuation_line() {
    let src = concat!(
        "module Faker\n",
        "  class Creature\n",
        "    class Bird < Base\n",
        "      class << self\n",
        "        def common_name(tax_order = nil)\n",
        "          if tax_order.nil?\n",
        "            sample\n",
        "          else\n",
        "            raise TypeError, 'must be symbolizable' \\\n",
        "              unless tax_order.respond_to?(:to_sym)\n",  // continuation — not a block opener
        "            raise ArgumentError, 'invalid order' \\\n",
        "                                 unless valid?(tax_order)\n",  // same
        "            the_order\n",
        "          end\n",
        "        end\n",
        "      end\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "backslash-continuation unless falsely flagged: {:?}", diags);
}

// ── Real misalignment must still be detected after `end,` patterns ────────────
#[test]
fn still_detects_misalignment_after_do_block_end_comma() {
    // The def is at indent 0 but its end is at indent 2 — that's a real violation.
    let src = concat!(
        "def foo\n",
        "  a.map do |x|\n",
        "    x\n",
        "  end,\n",     // end, for the do-block — should NOT be flagged
        "  x\n",
        "  end\n",      // misaligned: def at 0, end at 2
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned def end after end,, got none");
}

// `private_class_method def self.method_name` — the `def` is preceded by a
// modifier keyword. The `end` of this method must align with the `private_class_method`
// line's indent, not with the surrounding class/module indent.
#[test]
fn no_false_positive_for_private_class_method_def() {
    let src = concat!(
        "module Foo\n",
        "  private_class_method def self.bar(x)\n",
        "    x + 1\n",
        "  end\n",
        "\n",
        "  def self.baz\n",
        "    42\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`private_class_method def self.bar` must not be flagged, got: {:?}",
        diags
    );
}

// ── False positive: `do` block with trailing inline comment ───────────────────
// `foo do # :nodoc:` — the `do` is missed by `ends_with(" do")`, causing the
// do-block's `end` to pop the def frame instead of the do frame.
#[test]
fn no_false_positive_for_do_block_with_trailing_comment() {
    let src = concat!(
        "module MyEngine\n",
        "  def setup\n",
        "    ActiveSupport.on_load(:action_controller) do # :nodoc:\n",
        "      include Helper\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "do-block with trailing comment must not corrupt frame stack; got: {:?}",
        diags
    );
}

// ── False positive: one-liner keyword block `;end` (no space) ─────────────────
// `until x;end` — not caught by is_one_liner which only matches `"; end"`.
#[test]
fn no_false_positive_for_one_liner_keyword_block_no_space() {
    let src = concat!(
        "def outer\n",
        "  until scanner.eos?;end\n",
        "end\n",
        "\n",
        "def other\n",
        "  1\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "one-liner keyword block with ;end must not leave phantom frame; got: {:?}",
        diags
    );
}

// ── False positive: one-liner do-block ending with ` end` ─────────────────────
#[test]
fn no_false_positive_for_one_liner_do_block_space_end() {
    let src = concat!(
        "def outer\n",
        "  items.each do |x| process(x) end\n",
        "end\n",
        "\n",
        "def other\n",
        "  2\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "one-liner do-block ending with ' end' must not leave phantom frame; got: {:?}",
        diags
    );
}
