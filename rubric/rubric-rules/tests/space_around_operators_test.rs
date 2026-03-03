use rubric_core::{LintContext, Rule};
use rubric_rules::layout::space_around_operators::SpaceAroundOperators;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/space_around_operators/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/space_around_operators/corrected.rb");

#[test]
fn detects_missing_space_around_operators() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/SpaceAroundOperators"));
}

#[test]
fn no_violation_with_spaces_around_operators() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── False positive: ||= compound assignment ──────────────────────────────────
#[test]
fn no_false_positive_for_or_assign() {
    let src = "x ||= default\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "||= falsely flagged: {:?}", diags);
}

// ── False positive: &&= compound assignment ──────────────────────────────────
#[test]
fn no_false_positive_for_and_assign() {
    let src = "x &&= condition\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "&&= falsely flagged: {:?}", diags);
}

// ── False positive: ** double-splat in method parameter ──────────────────────
#[test]
fn no_false_positive_for_double_splat_param() {
    let src = "def foo(**opts)\n  opts\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "**opts in params falsely flagged: {:?}", diags);
}

// ── False positive: operators inside a regex literal ─────────────────────────
#[test]
fn no_false_positive_for_regex_operators() {
    let src = "pattern = /a+b*/\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "regex operators falsely flagged: {:?}", diags);
}

// ── True positive: ||= without spaces should still be detected ───────────────
#[test]
fn detects_missing_space_around_or_assign() {
    let src = "x||=default\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "x||=default should be flagged");
}

// ── No violation: ** exponentiation uses no_space style (RuboCop default) ────
#[test]
fn no_violation_for_exponentiation_operator() {
    let src = "x = 10**6\ny = a**b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for exponentiation, got: {:?}", diags);
}

// ── False positive: =~ regex match operator ───────────────────────────────────
#[test]
fn no_violation_for_regex_match_operator() {
    let src = "result = net =~ addr\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for =~ operator, got: {:?}", diags);
}

// ── False positive: setter method definition ──────────────────────────────────
#[test]
fn no_violation_for_setter_method_definition() {
    let src = "def bot=(val)\n  @bot = val\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for setter method def, got: {:?}", diags);
}

// ── False positive: setter method call (self.foo=val) ────────────────────────
#[test]
fn no_violation_for_setter_method_call() {
    let src = "self.discard_column=:deleted_at\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for setter method call, got: {:?}", diags);
}

// ── False positive: symbol literals like `:+`, `:-` (e.g., `reduce(:+)`) ────
#[test]
fn no_violation_for_symbol_operator_literal() {
    let src = "result = arr.reduce(:+)\nresult2 = arr.reduce(:-)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "symbol operator literals should not be flagged: {:?}", diags);
}

// ── False positive: %r{...} percent-regex content ────────────────────────────
#[test]
fn no_violation_for_percent_r_regex_content() {
    let src = "MENTION_RE = %r{(?<![=/[:word:]])@(([a-z0-9]+)(?:@[[:word:]]+)?)}i\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations inside %r{{}} regex, got: {:?}", diags);
}

// ── False positive: === (case equality) operator with proper spacing ──────────
#[test]
fn no_false_positive_for_case_equality_operator_with_spaces() {
    let src = "result = Integer === size\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "=== with spaces falsely flagged: {:?}", diags);
}

// ── True positive: === without spaces should be detected ─────────────────────
#[test]
fn detects_case_equality_operator_without_spaces() {
    let src = "result = Integer===size\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "Integer===size should be flagged");
}

// ── False positive: = in method parameter default (def foo(bar=nil)) ─────────
#[test]
fn no_false_positive_for_parameter_default() {
    let src = "def foo(bar=nil)\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "parameter default = falsely flagged: {:?}", diags);
}

// ── False positive: multiple parameter defaults ───────────────────────────────
#[test]
fn no_false_positive_for_multiple_parameter_defaults() {
    let src = "def configure(host='localhost', port=8080, ssl=false)\n  [host, port, ssl]\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "multiple parameter defaults falsely flagged: {:?}", diags);
}

// ── False positive: :[]]= symbol method literal ─────────────────────────────
#[test]
fn no_false_positive_for_bracket_assign_symbol() {
    let src = "def_delegators :@config, :[], :[]=, :fetch\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), ":[]]= symbol falsely flagged: {:?}", diags);
}

// ── False positive: :method= symbol (e.g., :config=, :bid=) ─────────────────
#[test]
fn no_false_positive_for_method_eq_symbol() {
    let src = "x.respond_to?(:config=)\nx.respond_to?(:bid=)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), ":method= symbol falsely flagged: {:?}", diags);
}

// ── False positive: splat `*` in block params `|*args|` ─────────────────────
#[test]
fn no_false_positive_for_splat_in_block_params() {
    let src = "define_method(name) do |*args, **kwargs|\n  @client.call(name, *args, **kwargs)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "|*args| splat falsely flagged: {:?}", diags);
}

// ── False positive: <<- heredoc sigil's `-` ──────────────────────────────────
#[test]
fn no_false_positive_for_heredoc_dash_sigil() {
    let src = concat!(
        "Action.class_eval <<-RUBY, filename, -1\n",
        "  def foo\n",
        "    bar\n",
        "  end\n",
        "RUBY\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "<<- heredoc sigil falsely flagged: {:?}", diags);
}

// ── False positive: operators inside <<~ heredoc body ────────────────────────
#[test]
fn no_false_positive_for_operators_in_heredoc_body() {
    let src = concat!(
        "def warn_msg\n",
        "  logger.warn <<~EOM\n",
        "    Your process is not CPU-saturated; reduce concurrency and/or\n",
        "    See: https://github.com/example/repo/wiki/Using-Redis#memory\n",
        "    REDIS_PROVIDER=REDISTOGO_URL\n",
        "  EOM\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "heredoc body operators falsely flagged: {:?}", diags);
}

// ── False positive: <=> spaceship operator ───────────────────────────────────
#[test]
fn no_false_positive_for_spaceship_operator() {
    let src = "sorted = arr.sort { |a, b| b <=> a }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "<=> spaceship falsely flagged: {:?}", diags);
}

// ── True positive: <=> without spaces should be detected ─────────────────────
#[test]
fn detects_spaceship_operator_without_spaces() {
    let src = "sorted = arr.sort { |a, b| b<=>a }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(!diags.is_empty(), "b<=>a should be flagged");
}

// ── False positive: operator symbols :<=, :>= ────────────────────────────────
#[test]
fn no_false_positive_for_operator_symbol() {
    let src = "assert_operator 1000, :<=, result\nassert_operator a, :>=, b\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "operator symbol :<=/:>= falsely flagged: {:?}", diags);
}

// ── False positive: %w[...] word array with paths/dashes ─────────────────────
#[test]
fn no_false_positive_for_percent_w_literal() {
    let src = "args = %w[sidekiq -r ./test/fake_env.rb]\nparts = %w[1-client-before 1-server-after]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "%w[] operators falsely flagged: {:?}", diags);
}

// ── False positive: %(string) percent string literal ─────────────────────────
#[test]
fn no_false_positive_for_percent_paren_literal() {
    let src = "html = %(<time class=\"ltr\" dir=\"ltr\">text</time>)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "%(string) operators falsely flagged: {:?}", diags);
}

// ── False positive: backtick shell string ────────────────────────────────────
#[test]
fn no_false_positive_for_backtick_string() {
    let src = "label = `git log -1 --format=\"%h %s\"`.strip\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "backtick string operators falsely flagged: {:?}", diags);
}

// ── False positive: =begin embedded doc delimiter ─────────────────────────────
#[test]
fn no_false_positive_for_equals_begin() {
    let src = concat!(
        "=begin\n",
        "This is an embedded documentation block.\n",
        "=end\n",
        "x = 1\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "=begin falsely flagged: {:?}", diags);
}

// ── False positive: =end embedded doc delimiter ───────────────────────────────
#[test]
fn no_false_positive_for_equals_end() {
    let src = "=end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "=end falsely flagged: {:?}", diags);
}

// ── False positive: / inside %w[] multiline word array ───────────────────────
#[test]
fn no_false_positive_for_slash_in_percent_w_multiline() {
    let src = concat!(
        "TEST_FILES = %w[\n",
        "  client_certs/ca.crt\n",
        "  client_certs/client.crt\n",
        "]\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "%w[] multiline slash falsely flagged: {:?}", diags);
}

// ── False positive: ?/ character literal ─────────────────────────────────────
#[test]
fn no_false_positive_for_question_slash_char_literal() {
    let src = "unless location[0] == ?/\n  do_something\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = SpaceAroundOperators.check_source(&ctx);
    assert!(diags.is_empty(), "?/ char literal falsely flagged: {:?}", diags);
}
