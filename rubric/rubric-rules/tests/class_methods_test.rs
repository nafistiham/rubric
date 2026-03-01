use rubric_core::{LintContext, Rule};
use rubric_rules::style::class_methods::ClassMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/class_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "class Foo\n  class << self\n    def bar\n      'bar'\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
