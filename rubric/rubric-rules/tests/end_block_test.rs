use rubric_core::{LintContext, Rule};
use rubric_rules::style::end_block::EndBlock;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/end_block/offending.rb");
const PASSING: &str = include_str!("fixtures/style/end_block/passing.rb");

#[test]
fn detects_end_block() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EndBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/EndBlock"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = EndBlock.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_correct_message() {
    let src = "END {\n  puts 'done'\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(
        diags[0].message.contains("Kernel#at_exit"),
        "message should mention Kernel#at_exit"
    );
}

#[test]
fn does_not_flag_lowercase_end() {
    let src = "end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndBlock.check_source(&ctx);
    assert!(diags.is_empty(), "lowercase end should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_end_block_without_space() {
    let src = "END{\n  puts 'done'\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndBlock.check_source(&ctx);
    assert!(!diags.is_empty(), "END{{ without space should be flagged");
}
