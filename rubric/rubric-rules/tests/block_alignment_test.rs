use rubric_core::{LintContext, Rule};
use rubric_rules::layout::block_alignment::BlockAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/block_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/BlockAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo do\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_do_block_inside_case() {
    // do-block nested inside case (non-do inner construct)
    let src = concat!(
        "foo.each do\n",
        "  case x\n",
        "  when :a\n",
        "    [1,2].each do |i|\n",
        "      puts i\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for do-block inside case, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_do_block_inside_if() {
    let src = concat!(
        "foo.each do\n",
        "  if cond\n",
        "    bar.each do |x|\n",
        "      puts x\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for do-block inside if, got: {:?}", diags);
}

#[test]
fn still_detects_misaligned_block_end() {
    let src = "foo.each do\n  bar\n    end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned block end, got none");
}

#[test]
fn no_false_positive_for_shovel_if_inline_conditional() {
    let src = "items.each do |x|\n  arr << if x.even?\n             x * 2\n           else\n             x\n           end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for << if inline conditional, got: {:?}", diags);
}

// ── FP fixes: begin...end expressions ────────────────────────────────────────

#[test]
fn no_fp_begin_end_inside_do_block() {
    // `val = begin ... rescue ... end` inside a do-block must not consume the do entry
    let src = concat!(
        "class_methods do\n",
        "  def my_method\n",
        "    val = begin\n",
        "      compute\n",
        "    rescue SomeError\n",
        "      nil\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for begin...end inside do-block, got: {:?}", diags);
}

#[test]
fn no_fp_begin_end_rhs_assignment() {
    // `result ||= begin ... end` pattern
    let src = concat!(
        "foo do\n",
        "  result ||= begin\n",
        "    expensive_call\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for ||= begin...end, got: {:?}", diags);
}

// ── FP fixes: end with trailing punctuation ───────────────────────────────────

#[test]
fn no_fp_end_with_trailing_comma() {
    // `end,` closes a block that is a hash value — must be recognised as an end statement
    let src = concat!(
        "configure do\n",
        "  use_all({\n",
        "    top: items.map do |x|\n",
        "      x * 2\n",
        "    end,\n",
        "  })\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for end followed by comma, got: {:?}", diags);
}

#[test]
fn no_fp_end_with_trailing_paren() {
    // `end)` closes a block passed as a method argument
    let src = concat!(
        "class_methods do\n",
        "  def my_method\n",
        "    result = foo.bar(items.flat_map do |x|\n",
        "      [x, x * 2]\n",
        "    end)\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for end followed by paren, got: {:?}", diags);
}

// ── FP fixes: endless method definitions ─────────────────────────────────────

#[test]
fn no_fp_endless_method_def_inside_do_block() {
    // `def obj.foo = expr` has no matching `end` and must not be pushed as inner construct
    let src = concat!(
        "after_build do |obj|\n",
        "  def obj.update_remote = true\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for endless method def inside do-block, got: {:?}", diags);
}

// ── FP fixes: method-chain do-block continuation ─────────────────────────────

#[test]
fn no_fp_method_chain_do_continuation() {
    // `do` is on a continuation line at deeper indent; `end` aligns with chain start
    let src = concat!(
        "    source_followers\n",
        "      .where(condition)\n",
        "      .in_batches do |batch|\n",
        "        batch.each { |x| x }\n",
        "    end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for method-chain do continuation, got: {:?}", diags);
}

// ── FP fixes: inline case on RHS of assignment ───────────────────────────────

#[test]
fn no_fp_inline_case_rhs() {
    // `c.name = case env ... end` — the case end must not consume the configure do entry
    let src = concat!(
        "configure do |c|\n",
        "  c.name = case env\n",
        "            when :prod then 'production'\n",
        "            else 'other'\n",
        "            end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for inline case on RHS, got: {:?}", diags);
}

#[test]
fn no_fp_nested_begin_inside_loop_do() {
    // `objects = begin ... rescue ... end` inside `loop do`
    let src = concat!(
        "loop do\n",
        "  objects = begin\n",
        "    fetch_objects\n",
        "  rescue => e\n",
        "    break\n",
        "  end\n",
        "  break if objects.empty?\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for begin...end inside loop do, got: {:?}", diags);
}

// ── FP fixes: single-line and setter defs ────────────────────────────────────

#[test]
fn no_fp_single_line_def_inside_do_block() {
    // `def foo; body; end` is a single-line def and must not push a stack entry
    let src = concat!(
        "Class.new do\n",
        "  def self.account(service, username); end\n",
        "  def hoge=(arg); end\n",
        "  def hoge_file_name; end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for single-line defs inside do-block, got: {:?}", diags);
}

#[test]
fn no_fp_setter_method_def_inside_do_block() {
    // `def foo=(arg)` is a setter method — NOT an endless def — and has a matching `end`
    let src = concat!(
        "included do\n",
        "  def expires_in=(interval)\n",
        "    self.expires_at = interval\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for setter def inside do-block, got: {:?}", diags);
}

#[test]
fn no_fp_do_block_end_aligns_with_do_line_not_chain_start() {
    // When `do` is on a continuation line, `end` may align with the `do` line indent
    // rather than the chain start — both are acceptable.
    let src = concat!(
        "    expect { subject.perform }\n",
        "      .to enqueue_sidekiq_job(Worker)\n",
        "      .with(satisfying do |body|\n",     // .with is at indent 6; chain start at 4
        "        body == 'ok'\n",
        "      end, recipient.id)\n",             // end aligns with .with line (indent 6)
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP when end aligns with do-line indent, got: {:?}", diags);
}
