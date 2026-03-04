use rubric_core::{LintContext, Rule};
use rubric_rules::layout::else_alignment::ElseAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/else_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/ElseAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "if foo\n  bar\nelse\n  baz\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── FP: `else` inside a `do` block that itself is inside an `if` ─────────────
// The `end` for the inner `do` block must not prematurely pop the `if` frame,
// causing the outer `else` to be flagged.
#[test]
fn no_false_positive_for_else_after_do_block_inside_if() {
    let src = concat!(
        "def require_challenge!\n",
        "  if params.key?(:form)\n",
        "    if challenge_passed?\n",
        "      update_session\n",
        "    else\n",
        "      render_challenge\n",
        "    end\n",
        "  else\n",
        "    render_challenge\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "else after nested if falsely flagged: {:?}", diags);
}

// ── FP: `else` inside a `case` block must not be checked by ElseAlignment ────
// ElseAlignment only checks `else`/`elsif` that belong to `if`/`unless`.
#[test]
fn no_false_positive_for_else_in_case_block() {
    let src = concat!(
        "def color(processed, failed)\n",
        "  case\n",
        "  when !processed.zero? && failed.zero?\n",
        "    :green\n",
        "  elsif failed.zero?\n",
        "    :yellow\n",
        "  else\n",
        "    :red\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "else in case falsely flagged: {:?}", diags);
}

// ── FP: `else` inside inline `case` assigned to a variable ───────────────────
// Pattern: `x = case y ... when ... else ... end`
// The `else` belongs to the inline `case`, not to any outer `if`.
#[test]
fn no_false_positive_for_else_in_inline_case() {
    let src = concat!(
        "if ENV.present?\n",
        "  c.service_name =  case $PROGRAM_NAME\n",
        "                    when /puma/ then 'web'\n",
        "                    else\n",
        "                      'other'\n",
        "                    end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "else in inline case (double-space) falsely flagged: {:?}", diags);
}

// ── FP: unified stack — `if` inside `def` with multiple `do` blocks ──────────
// The two-counter approach fails when `end` tokens for `def`/`do` interleave
// with `end` tokens for `if`. The unified stack must pop correctly.
#[test]
fn no_false_positive_for_if_inside_def_with_do_blocks() {
    let src = concat!(
        "def add_to_feed(aggregate: true)\n",
        "  if status.reblog? && aggregate\n",
        "    if redis.zadd(key, status.id, status.id)\n",
        "      redis.zadd(timeline_key, status.id, status.id)\n",
        "    else\n",
        "      redis.sadd(reblog_key, status.id)\n",
        "      return false\n",
        "    end\n",
        "  else\n",
        "    return false unless redis.zscore(key, status.id).nil?\n",
        "    redis.zadd(timeline_key, status.id, status.id)\n",
        "  end\n",
        "  true\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "if inside def with do blocks falsely flagged: {:?}", diags);
}

// ── FP: inline `= if` where `else` aligns with the `if` keyword column ───────
// Pattern: `@domain = if condition` — `else` at the column of `if`.
#[test]
fn no_false_positive_for_inline_if_else_alignment() {
    // "    @domain = if ..." — `if` is at column 14 (0-indexed).
    // `else` must be at the same column (14 spaces of indent).
    let src = concat!(
        "def process!(uri)\n",
        "    @domain = if local_domain?(@domain)\n",
        "                 nil\n",
        "              else\n",
        "                 normalize(@domain)\n",
        "              end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "inline = if else alignment falsely flagged: {:?}", diags);
}

// ── FP: `else` inside deeply nested if/do, stale state from outer context ────
#[test]
fn no_false_positive_for_stale_stack_state_across_methods() {
    // Multiple methods with `do` blocks that contain `if` — the ElseAlignment
    // stack must be kept consistent so that later methods aren't affected.
    let src = concat!(
        "def method_one\n",
        "  items.each do |x|\n",
        "    if x.valid?\n",
        "      process(x)\n",
        "    end\n",
        "  end\n",
        "end\n",
        "\n",
        "def method_two\n",
        "  if condition\n",
        "    do_work\n",
        "  else\n",
        "    skip\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ElseAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "stale stack state across methods falsely flagged: {:?}", diags);
}
