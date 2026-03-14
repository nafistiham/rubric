use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_method_definition::UselessMethodDefinition;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_method_definition/offending.rb");
const PASSING: &str =
    include_str!("fixtures/lint/useless_method_definition/passing.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessMethodDefinition.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessMethodDefinition"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = UselessMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on passing.rb; got: {:?}", diags);
}

#[test]
fn detects_multiple_useless_definitions() {
    // Both foo and bar are useless — only `super` in body
    let src = "def foo\n  super\nend\n\ndef bar(x, y)\n  super\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessMethodDefinition.check_source(&ctx);
    assert_eq!(diags.len(), 2, "expected 2 violations; got: {:?}", diags);
}

#[test]
fn no_violation_when_body_has_additional_work() {
    let src = "def foo\n  super\n  do_extra_work\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessMethodDefinition.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
}
