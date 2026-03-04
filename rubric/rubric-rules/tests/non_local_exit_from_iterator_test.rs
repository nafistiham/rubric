use rubric_core::{LintContext, Rule};
use rubric_rules::lint::non_local_exit_from_iterator::NonLocalExitFromIterator;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/non_local_exit_from_iterator/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/NonLocalExitFromIterator"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2].each do |x|\n  next if x > 1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// `return` inside a lambda do...end is a local exit from the lambda, NOT
// a non-local exit from the enclosing method.
#[test]
fn no_false_positive_for_return_in_lambda_do_block() {
    let src = concat!(
        "TRANSFORMER = lambda do |env|\n",
        "  node = env[:node]\n",
        "  return unless node\n",
        "  node.remove_attribute('x')\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(diags.is_empty(), "return in lambda do block falsely flagged: {:?}", diags);
}

// `return` inside define_method do...end is a return from the defined method.
#[test]
fn no_false_positive_for_return_in_define_method_do_block() {
    let src = concat!(
        "KEYS.each do |key|\n",
        "  define_method(key) do\n",
        "    return instance_variable_get(:\"@#{key}\") if instance_variable_defined?(:\"@#{key}\")\n",
        "    Setting.public_send(key)\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    // `return` inside define_method block is a method-local return, not an iterator exit
    assert!(diags.is_empty(), "return in define_method do block falsely flagged: {:?}", diags);
}

// `return` inside a method that has a begin/rescue/end block — the begin's `end`
// must not decrement def_depth prematurely and trigger a false block detection.
#[test]
fn no_false_positive_for_return_after_begin_rescue_inside_def() {
    // Pattern from mastodon account/field.rb: extract_url_from_html
    // The `begin...rescue...end` inside a method was making def_depth drop to 0,
    // causing subsequent `return if` lines to be falsely flagged.
    let src = concat!(
        "class Field\n",
        "  def extract_url_from_html\n",
        "    begin\n",
        "      doc = parse(value)\n",
        "    rescue ArgumentError\n",
        "      return\n",
        "    end\n",
        "\n",
        "    return if doc.nil?\n",
        "    return if doc.children.size != 1\n",
        "    doc.children.first\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(diags.is_empty(), "return after begin/rescue inside def falsely flagged: {:?}", diags);
}

// `t.contains(" do")` must not match words like "doc", "domain", "done" etc.
// The check needs word-boundary awareness for the "do" token.
#[test]
fn no_false_positive_for_do_substring_in_variable_names() {
    // "doc", "domain", etc contain " do" as a substring but are NOT block openers.
    let src = concat!(
        "class Field\n",
        "  def extract\n",
        "    return if doc.nil?\n",
        "    return if domain.blank?\n",
        "    doc.children.first\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(diags.is_empty(), "do-substring in var names falsely flagged: {:?}", diags);
}

// `return` inside a def that is inside a class at top-level (def_depth > 0)
// must not be flagged even if block_depth is manipulated incorrectly.
#[test]
fn no_false_positive_for_return_inside_method_with_do_word_in_code() {
    // Simulates: validates :acct, domain: { acct: true }  <- "domain" contains "do"
    // followed by a def with a return
    let src = concat!(
        "class AccountMigration\n",
        "  validates :acct, domain: { acct: true }\n",
        "\n",
        "  def save_with_challenge(current_user)\n",
        "    return false unless errors.empty?\n",
        "    with_redis_lock(\"key\") do\n",
        "      save\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NonLocalExitFromIterator.check_source(&ctx);
    assert!(diags.is_empty(), "return in method with domain: attribute falsely flagged: {:?}", diags);
}
