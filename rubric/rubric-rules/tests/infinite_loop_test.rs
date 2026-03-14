use rubric_core::{LintContext, Rule};
use rubric_rules::style::infinite_loop::InfiniteLoop;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/infinite_loop/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/infinite_loop/clean.rb");

#[test]
fn detects_infinite_loops() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/InfiniteLoop"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_while_true() {
    let src = "while true\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "while true should be flagged");
    assert!(diags[0].message.contains("loop"));
}

#[test]
fn detects_until_false() {
    let src = "until false\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "until false should be flagged");
}

#[test]
fn detects_while_true_do() {
    let src = "while true do\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(!diags.is_empty(), "while true do should be flagged");
}

#[test]
fn does_not_flag_while_condition() {
    let src = "while condition\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(diags.is_empty(), "while condition should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_until_done() {
    let src = "until done?\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(diags.is_empty(), "until done? should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment() {
    let src = "# while true\n# until false\nloop do\n  foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(diags.is_empty(), "comments should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = r#"msg = "use while true for infinite loops"
puts msg
"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = InfiniteLoop.check_source(&ctx);
    assert!(diags.is_empty(), "string contents should not be flagged, got: {:?}", diags);
}
