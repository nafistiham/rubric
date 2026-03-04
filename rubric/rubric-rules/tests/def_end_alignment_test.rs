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
