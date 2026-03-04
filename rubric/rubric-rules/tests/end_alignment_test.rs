use rubric_core::{LintContext, Rule};
use rubric_rules::layout::end_alignment::EndAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/end_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/end_alignment/corrected.rb");

#[test]
fn detects_misaligned_end() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EndAlignment"));
}

#[test]
fn no_violation_for_aligned_end() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_inline_if_assignment() {
    let src = "module Foo\n  class Bar\n    def email\n      local_part = if true\n                     'a'\n                   else\n                     'b'\n                   end\n      local_part\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for inline if assignment, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_end_dot_method_chain() {
    let src = "def foo\n  [1, 2, 3].map do |x|\n    x + 1\n  end.join(', ')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for end.method chain, got: {:?}", diags);
}

#[test]
fn still_detects_misaligned_end_after_def() {
    let src = "def foo\n  1\n    end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned end, got none");
}

#[test]
fn no_false_positive_for_shovel_if_inline_conditional() {
    let src = "def foo\n  arr << if cond\n             val1\n           else\n             val2\n           end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for << if inline conditional, got: {:?}", diags);
}

// ── False positive: `||= if` compound assignment followed by if ──────────────
#[test]
fn no_false_positive_for_or_assign_inline_if() {
    let src = concat!(
        "def display_args\n",
        "  @cache ||= if cond1\n",
        "    if cond2\n",
        "      val\n",
        "    end\n",
        "    args\n",
        "  else\n",
        "    args\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "||= if falsely flagged: {:?}", diags);
}

// ── False positive: `||= ... || begin` inline begin with nested if ────────────
#[test]
fn no_false_positive_for_inline_or_begin_with_nested_if() {
    let src = concat!(
        "def display_class\n",
        "  @klass ||= self['x'] || begin\n",
        "    if cond1\n",
        "      if cond2\n",
        "        args[0]\n",
        "      else\n",
        "        val\n",
        "      end\n",
        "    else\n",
        "      klass\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "|| begin with nested if falsely flagged: {:?}", diags);
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
    let diags = EndAlignment.check_source(&ctx);
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
    let diags = EndAlignment.check_source(&ctx);
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
    let diags = EndAlignment.check_source(&ctx);
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
    let diags = EndAlignment.check_source(&ctx);
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
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "protected def falsely flagged: {:?}", diags);
}

// ── False positive: inline `if` + nested inline `begin` assignment ────────────
// Pattern: val = if cond / job = begin ... rescue ... end / ... / else / end
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
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "nested inline begin in inline if falsely flagged: {:?}", diags);
}

// ── False positive: `end,` inside a hash/array (e.g., `top_hashtags.map do ... end,`) ──
// `end` followed by `,` should be recognized as a valid `end` token and pop the stack.
#[test]
fn no_false_positive_for_end_comma_in_hash() {
    let src = concat!(
        "class Report\n",
        "  def generate\n",
        "    {\n",
        "      items: list.map do |(name, count)|\n",
        "               { name: name, count: count }\n",
        "             end,\n",
        "    }\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end, in hash falsely flagged: {:?}", diags);
}

// ── False positive: `end)` inside a method call argument ─────────────────────
// `end` followed by `)` should be recognized as a valid `end` token.
#[test]
fn no_false_positive_for_end_paren() {
    let src = concat!(
        "def foo\n",
        "  result = bar(list.each_with_object([]) do |item, acc|\n",
        "    acc << item\n",
        "  end).compact\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end) falsely flagged: {:?}", diags);
}

// ── False positive: `do` block `end` — EndAlignment does not check do-block ends ──
// Rubocop's Layout/EndAlignment skips do-block end alignment (that's BlockAlignment).
// `end` for a `do` block should never generate an EndAlignment diagnostic.
#[test]
fn no_false_positive_for_do_block_end_misaligned_style() {
    // In this style, `end` aligns with the expression start (source_list),
    // not the `do` keyword column.
    let src = concat!(
        "def rewrite!\n",
        "  source_list\n",
        "    .where(active: true)\n",
        "    .in_batches do |batch|\n",
        "      process(batch)\n",
        "    end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "do-block end alignment falsely flagged: {:?}", diags);
}

// ── False positive: multiple `do` blocks with the class/module closer ────────
// Many `do` blocks in a file must not corrupt the stack so that
// the class/module `end` is flagged.
#[test]
fn no_false_positive_for_multiple_do_blocks() {
    let src = concat!(
        "module BulkConcern\n",
        "  def push_bulk(args_array)\n",
        "    Client.push_bulk({\n",
        "      'args' => args_array.map do |args|\n",
        "        [args]\n",
        "      end,\n",
        "    })\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "multiple do blocks falsely flagged: {:?}", diags);
}

// ── False positive: heredoc body containing `end` (e.g., SQL CASE ... END) ───
#[test]
fn no_false_positive_for_end_in_heredoc_body() {
    let src = concat!(
        "module Search\n",
        "  SQL = <<~SQL.squish\n",
        "    SELECT\n",
        "      case when x IS NULL then 0\n",
        "           else 1\n",
        "      end\n",
        "    FROM accounts\n",
        "  SQL\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end inside heredoc body falsely flagged: {:?}", diags);
}

// ── False positive: `private_class_method def` opener ────────────────────────
#[test]
fn no_false_positive_for_private_class_method_def() {
    let src = concat!(
        "module MyHelper\n",
        "  private_class_method def self.helper_method(x)\n",
        "    x * 2\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "private_class_method def falsely flagged: {:?}", diags);
}

// ── False positive: inline `case` with extra whitespace `=  case` ────────────
#[test]
fn no_false_positive_for_inline_case_double_space() {
    let src = concat!(
        "def name\n",
        "  result =  case @type\n",
        "             when :a then 'foo'\n",
        "             else 'bar'\n",
        "             end\n",
        "  result\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline case with double space falsely flagged: {:?}", diags);
}

// ── False positive: `end:` hash key (e.g., `end: 5,`) is NOT an `end` token ──
// A hash key `end:` should not be treated as a Ruby `end` keyword.
#[test]
fn no_false_positive_for_end_colon_hash_key() {
    let src = concat!(
        "class X < Base\n",
        "  class << self\n",
        "    def tweet_entities\n",
        "      {\n",
        "        urls: [\n",
        "          {\n",
        "            start: 0,\n",
        "            end: 5,\n",
        "            url: url\n",
        "          }\n",
        "        ],\n",
        "        hashtags: [\n",
        "          {\n",
        "            start: 0,\n",
        "            end: 5,\n",
        "            tag: 'foo'\n",
        "          }\n",
        "        ]\n",
        "      }\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "end: hash key falsely flagged: {:?}", diags);
}

// ── False positive: multiline `raise` with `unless` on the continuation line ──
// When a `raise` spans two lines with `\` continuation, the second line
// starting with `unless` is a continuation, not a new `unless` block opener.
#[test]
fn no_false_positive_for_unless_on_continuation_line() {
    let src = concat!(
        "module Faker\n",
        "  class Bird < Base\n",
        "    class << self\n",
        "      def common_name(tax_order = nil)\n",
        "        if tax_order.nil?\n",
        "          sample_value\n",
        "        else\n",
        "          raise TypeError, 'must be symbolizable' \\\n",
        "            unless tax_order.respond_to?(:to_sym)\n",
        "          translate(tax_order.to_sym)\n",
        "        end\n",
        "      end\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "unless on continuation line falsely flagged: {:?}", diags);
}

// ── False positive: inline `if` after arithmetic operator (`acc + if`) ────────
// A do block containing `acc + if cond` has its `if` as an inline
// expression that produces an `end`; that `end` must not consume the do frame.
#[test]
fn no_false_positive_for_inline_if_after_operator_in_do_block() {
    let src = concat!(
        "def cif_valid?(cif)\n",
        "  if cif =~ regex\n",
        "    total = chars.inject(0) do |acc, (el, idx)|\n",
        "      acc + if idx.even?\n",
        "               (el.to_i * 2).digits.inject(:+)\n",
        "             else\n",
        "               el.to_i\n",
        "             end\n",
        "    end\n",
        "    total\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline if after operator in do block falsely flagged: {:?}", diags);
}
