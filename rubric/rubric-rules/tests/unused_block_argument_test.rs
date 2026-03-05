use rubric_core::{LintContext, Rule};
use rubric_rules::lint::unused_block_argument::UnusedBlockArgument;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/unused_block_argument/offending.rb");

fn check(src: &str) -> Vec<rubric_core::Diagnostic> {
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![Box::new(UnusedBlockArgument)];
    let mut diags: Vec<_> = rules.iter().flat_map(|r| r.check_source(&ctx)).collect();
    diags.extend(rubric_core::walk(src.as_bytes(), &ctx, &rules));
    diags
}

#[test]
fn detects_violation() {
    let diags = check(OFFENDING);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UnusedBlockArgument"));
}

#[test]
fn no_violation_on_clean() {
    let src = "[1, 2].each do |x|\n  puts x\nend\n";
    let diags = check(src);
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
    let diags = check(src);
    assert!(diags.is_empty(), "destructured params falsely flagged: {:?}", diags);
}

// _-prefixed params must never be flagged
#[test]
fn no_false_positive_for_underscore_prefixed_block_param() {
    let src = "[1, 2].each do |_unused|\n  puts 'hi'\nend\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no FP for _-prefixed block param, got: {:?}", diags);
}

// Brace block { |x| ... } — must be detected (not only do/end blocks)
#[test]
fn detects_unused_in_brace_block() {
    let src = "[1, 2].map { |x| 'const' }\n";
    let diags = check(src);
    assert!(!diags.is_empty(), "expected violation for unused param in brace block");
    assert!(diags.iter().any(|d| d.message.contains('x')));
}

// Brace block with used param — no violation
#[test]
fn no_violation_in_brace_block_with_used_param() {
    let src = "[1, 2].map { |x| x * 2 }\n";
    let diags = check(src);
    assert!(diags.is_empty(), "expected no violation when brace block param is used, got: {:?}", diags);
}

// Multi-param block — only the unused ones are flagged
#[test]
fn flags_only_unused_params_in_multi_param_block() {
    let src = "[1, 2].each_with_index do |item, index|\n  puts item\nend\n";
    let diags = check(src);
    assert_eq!(diags.len(), 1, "expected exactly 1 violation (index unused), got: {:?}", diags);
    assert!(diags[0].message.contains("index"));
}
