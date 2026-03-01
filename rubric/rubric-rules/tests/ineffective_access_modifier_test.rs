use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ineffective_access_modifier::IneffectiveAccessModifier;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/ineffective_access_modifier/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = IneffectiveAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/IneffectiveAccessModifier"));
}

#[test]
fn no_violation_on_clean() {
    let src = "class Foo\n  def self.bar\n    'bar'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = IneffectiveAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
