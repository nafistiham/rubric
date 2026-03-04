use rubric_core::{LintContext, Rule};
use rubric_rules::lint::constant_definition_in_block::ConstantDefinitionInBlock;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/constant_definition_in_block/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/ConstantDefinitionInBlock"));
}

#[test]
fn no_violation_on_clean() {
    let src = "FOO = 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Regression: constant in class body with `before_action -> { doorkeeper_... }` must not be flagged
// The word `doorkeeper` starts with `do` and was incorrectly matched as a `do` block opener.
#[test]
fn no_fp_constant_in_class_body_with_doorkeeper() {
    let src = concat!(
        "class Foo < Bar\n",
        "  before_action -> { doorkeeper_authorize! :read }, only: [:index]\n",
        "\n",
        "  MY_CONSTANT = 42\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "constant in class body must not be flagged when doorkeeper appears in before_action, got: {:?}",
        diags
    );
}

// Regression: constant at class level preceded by a line with a `do`-word (not a block)
#[test]
fn no_fp_constant_in_class_with_domain_word() {
    let src = concat!(
        "class Foo\n",
        "  validates :domain, presence: true\n",
        "\n",
        "  DOMAIN_LIMIT = 255\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "constant in class body must not be flagged when `domain` appears on a prior line, got: {:?}",
        diags
    );
}

// Real violation: constant inside an `each do` block must still be flagged
#[test]
fn detects_constant_in_each_do_block() {
    let src = concat!(
        "[1, 2].each do\n",
        "  FOO = 1\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "constant inside each-do block must be flagged"
    );
}

// Real violation: constant inside a `describe do |var|` block must still be flagged
#[test]
fn detects_constant_in_describe_do_block() {
    let src = concat!(
        "describe Foo do |var|\n",
        "  BAR = 2\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "constant inside describe-do block must be flagged"
    );
}

// Regression: constant assigned to a lambda/proc with `do...end` must NOT be flagged
// e.g. `MY_TRANSFORMER = lambda do |env| ... end`
#[test]
fn no_fp_constant_assigned_to_lambda_do_block() {
    let src = concat!(
        "module Config\n",
        "  MY_TRANSFORMER = lambda do |env|\n",
        "    env[:node].remove_attribute('class')\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "constant assigned to lambda do-block must not be flagged, got: {:?}",
        diags
    );
}

// Regression: constant-like text inside a heredoc body must NOT be flagged
#[test]
fn no_fp_constant_pattern_inside_heredoc() {
    let src = concat!(
        "class Foo\n",
        "  def greet\n",
        "    say(<<~MSG)\n",
        "      SELF_DESTRUCT=enabled\n",
        "    MSG\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "constant-like text inside heredoc body must not be flagged, got: {:?}",
        diags
    );
}

// Regression: `do` keyword in natural language inside a heredoc must not inflate block_depth
#[test]
fn no_fp_do_word_in_heredoc_body() {
    let src = concat!(
        "class Foo\n",
        "  def run\n",
        "    puts(<<~WARN)\n",
        "      Please do not shut it down until it has finished.\n",
        "    WARN\n",
        "\n",
        "    MY_CONST = 42\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    // `MY_CONST = 42` is inside `def run` (not a block), so no violation.
    let diags = ConstantDefinitionInBlock.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`do` in heredoc body must not inflate block depth, got: {:?}",
        diags
    );
}
