use rubric_core::{LintContext, Rule};
use rubric_rules::style::module_function::ModuleFunction;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/module_function/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ModuleFunction.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ModuleFunction"));
}

#[test]
fn no_violation_on_clean() {
    let src = "module Foo\n  module_function :bar\n  def bar\n    'bar'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ModuleFunction.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
