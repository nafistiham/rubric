use rubric_core::{LintContext, Rule};
use rubric_rules::lint::deprecated_class_methods::DeprecatedClassMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/deprecated_class_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DeprecatedClassMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/DeprecatedClassMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "File.exist?('foo.txt')\nDir.exist?('/tmp')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DeprecatedClassMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
