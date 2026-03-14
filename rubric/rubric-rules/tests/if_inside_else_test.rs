use rubric_core::{LintContext, Rule};
use rubric_rules::style::if_inside_else::IfInsideElse;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/if_inside_else/offending.rb");
const PASSING: &str = include_str!("fixtures/style/if_inside_else/passing.rb");

#[test]
fn detects_if_inside_else() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IfInsideElse.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/IfInsideElse"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = IfInsideElse.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_correct_message() {
    let src = "if x\n  a\nelse\n  if y\n    b\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IfInsideElse.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation");
    assert!(
        diags[0].message.contains("elsif"),
        "message should mention elsif"
    );
}

#[test]
fn does_not_flag_elsif() {
    let src = "if x\n  a\nelsif y\n  b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IfInsideElse.check_source(&ctx);
    assert!(diags.is_empty(), "elsif should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_else_with_non_if_content() {
    let src = "if x\n  a\nelse\n  b\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IfInsideElse.check_source(&ctx);
    assert!(diags.is_empty(), "else with non-if body should not be flagged, got: {:?}", diags);
}
