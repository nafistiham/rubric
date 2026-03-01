use rubric_core::{LintContext, Rule};
use rubric_rules::lint::struct_new_override::StructNewOverride;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/struct_new_override/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = StructNewOverride.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/StructNewOverride"));
}

#[test]
fn no_violation_on_clean() {
    let src = "Foo = Struct.new(:bar) do\n  def hello\n    'hello'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = StructNewOverride.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
