use rubric_core::{LintContext, Rule};
use rubric_rules::lint::top_level_return_with_argument::TopLevelReturnWithArgument;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/top_level_return_with_argument/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/TopLevelReturnWithArgument"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def foo\n  return 1\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── False positive: `return` after inline conditional assignment inside `def` ─
// Lines like `result = if cond` open an `if` block that doesn't start with
// "if " (because of the LHS assignment), so they must be tracked separately.
// The matching `end` should decrement block_depth, not def_depth.
#[test]
fn no_false_positive_for_return_after_inline_if_assignment() {
    // Without a class wrapper so the inline-if `end` directly threatens def_depth.
    let src = concat!(
        "def bar(account)\n",
        "  result = if account.nil?\n",
        "               nil\n",
        "             else\n",
        "               account.name\n",
        "             end\n",
        "\n",
        "  return '-' if result.nil?\n",
        "  return 'nope' unless result\n",
        "\n",
        "  result\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "return after inline if assignment inside def falsely flagged: {:?}",
        diags
    );
}

// Same as above but with a surrounding class (mirrors mastodon dashboard_helper.rb).
#[test]
fn no_false_positive_for_return_after_inline_if_assignment_in_class() {
    let src = concat!(
        "class Foo\n",
        "  def bar(account)\n",
        "    result = if account.nil?\n",
        "               nil\n",
        "             else\n",
        "               account.name\n",
        "             end\n",
        "\n",
        "    return '-' if result.nil?\n",
        "    return 'nope' unless result\n",
        "\n",
        "    result\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "return after inline if assignment inside class def falsely flagged: {:?}",
        diags
    );
}

// ── False positive: `return` after `loop do...end` inside a `def` ────────────
// The `end` closing a `loop do` block must not reduce def depth.
#[test]
fn no_false_positive_for_return_after_loop_block_inside_def() {
    let src = concat!(
        "def username(specifier: nil)\n",
        "  loop do\n",
        "    result = compute\n",
        "    break if done\n",
        "  end\n",
        "  return result * 2 if specifier.positive?\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TopLevelReturnWithArgument.check_source(&ctx);
    assert!(diags.is_empty(), "return after loop block inside def falsely flagged: {:?}", diags);
}
