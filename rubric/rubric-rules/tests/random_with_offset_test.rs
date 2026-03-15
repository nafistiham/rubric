use rubric_core::{LintContext, Rule};
use rubric_rules::style::random_with_offset::RandomWithOffset;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/random_with_offset/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/random_with_offset/corrected.rb");

#[test]
fn detects_rand_with_offset() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RandomWithOffset.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/RandomWithOffset"));
}

#[test]
fn detects_all_three_violations() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RandomWithOffset.check_source(&ctx);
    assert_eq!(diags.len(), 3, "expected 3 violations, got: {:?}", diags);
}

#[test]
fn no_violation_on_range_literal() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RandomWithOffset.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on corrected code, got: {:?}", diags);
}

#[test]
fn no_violation_on_plain_rand() {
    let src = "x = rand(10)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RandomWithOffset.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on plain rand, got: {:?}", diags);
}

#[test]
fn detects_rand_with_spaces_around_plus() {
    let src = "rand(5)  +  3\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RandomWithOffset.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation with spaces around +");
}

#[test]
fn no_violation_in_comment() {
    let src = "# rand(6) + 1 is bad\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RandomWithOffset.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation in comment, got: {:?}", diags);
}
