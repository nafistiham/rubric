use rubric_core::{LintContext, Rule};
use rubric_rules::style::map_to_hash::MapToHash;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/map_to_hash/offending.rb");
const CORRECTED: &str = include_str!("fixtures/style/map_to_hash/corrected.rb");

#[test]
fn detects_map_to_h_pattern() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MapToHash.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `.map {{ }}.to_h`, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/MapToHash"));
}

#[test]
fn no_violation_for_to_h_block() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MapToHash.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for `.to_h {{ }}`, got: {:?}", diags);
}

#[test]
fn detects_inline_map_braces_to_h() {
    let src = "result = arr.map { |x| [x, x.to_s] }.to_h\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MapToHash.check_source(&ctx);
    assert!(!diags.is_empty(), "inline `.map {{ }}.to_h` should be flagged");
    assert_eq!(diags[0].rule, "Style/MapToHash");
}

#[test]
fn no_violation_for_plain_to_h() {
    // .to_h without preceding map block is fine
    let src = "hash = something.to_h\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MapToHash.check_source(&ctx);
    assert!(diags.is_empty(), "plain `.to_h` should not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_to_h_with_args() {
    // .to_h with arguments is not the pattern we flag
    let src = "hash = pairs.to_h { |k, v| [k.to_s, v] }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MapToHash.check_source(&ctx);
    assert!(diags.is_empty(), "`.to_h {{ }}` (already correct) should not be flagged, got: {:?}", diags);
}

#[test]
fn detects_multiline_map_do_end_to_h() {
    let src = "result = arr.map do |x|\n  [x, x.to_s]\nend.to_h\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MapToHash.check_source(&ctx);
    assert!(!diags.is_empty(), "multiline `.map do...end.to_h` should be flagged");
}

#[test]
fn no_violation_for_comment_line() {
    let src = "# array.map { |x| [x, x] }.to_h\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MapToHash.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}
