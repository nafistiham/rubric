use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unused_block_argument::UnusedBlockArgument;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/unused_block_argument/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UnusedBlockArgument.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnusedBlockArgument"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2].each do |x|\n  puts x\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedBlockArgument.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// Destructuring block params like `|sum, (value, index)|` — `(value` and
// `index)` must not be treated as raw argument names
#[test]
fn no_false_positive_for_destructured_block_params() {
    let src = concat!(
        "control = code.chars.each_with_index.inject(0) do |sum, (value, index)|\n",
        "  if (index + 1).even?\n",
        "    sum + value.to_i\n",
        "  else\n",
        "    sum + algo(value.to_i)\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UnusedBlockArgument.check_source(&ctx);
    assert!(diags.is_empty(), "destructured params falsely flagged: {:?}", diags);
}
