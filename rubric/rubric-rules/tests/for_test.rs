use rubric_core::{LintContext, Rule};
use rubric_rules::style::for_loop::ForLoop;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/for/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/for/clean.rb");

#[test]
fn detects_for_loop() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ForLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/For"));
}

#[test]
fn no_violation_for_each() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = ForLoop.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_basic_for_in() {
    let src = "for i in 1..10\n  puts i\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ForLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "for i in range should be flagged");
    assert!(diags[0].message.contains("each"));
}

#[test]
fn flags_for_with_array() {
    let src = "for item in array\n  process(item)\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ForLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "for item in array should be flagged");
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# for i in 1..10\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ForLoop.check_source(&ctx);
    assert!(diags.is_empty(), "comment line should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_each_loop() {
    let src = "(1..10).each do |i|\n  puts i\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ForLoop.check_source(&ctx);
    assert!(diags.is_empty(), "each loop should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_for_in_string() {
    let src = "msg = \"for i in range\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ForLoop.check_source(&ctx);
    assert!(diags.is_empty(), "for in a string should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_indented_for() {
    let src = "  for x in collection\n    use(x)\n  end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ForLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "indented for should be flagged");
}

#[test]
fn detects_correct_count_of_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ForLoop.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations for 2 for loops, got {}", diags.len());
}
