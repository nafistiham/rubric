use rubric_core::{LintContext, Rule};
use rubric_rules::style::struct_inheritance::StructInheritance;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/struct_inheritance/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StructInheritance.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/StructInheritance"));
}

#[test]
fn no_violation_on_clean() {
    let src = "Foo = Struct.new(:bar, :baz)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StructInheritance.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
