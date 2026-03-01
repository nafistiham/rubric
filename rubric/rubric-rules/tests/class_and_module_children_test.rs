use rubric_core::{LintContext, Rule};
use rubric_rules::style::class_and_module_children::ClassAndModuleChildren;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/class_and_module_children/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassAndModuleChildren"));
}

#[test]
fn no_violation_on_clean() {
    let src = "class Foo\n  class Bar\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
