use rubric_core::{LintContext, Rule};
use rubric_rules::style::access_modifier_declarations::AccessModifierDeclarations;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/access_modifier_declarations/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = AccessModifierDeclarations.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/AccessModifierDeclarations"));
}

#[test]
fn no_violation_on_clean() {
    let src = "class Foo\n  private\n  def bar\n    'bar'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = AccessModifierDeclarations.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
