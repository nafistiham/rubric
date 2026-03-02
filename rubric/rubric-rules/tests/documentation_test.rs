use rubric_core::{LintContext, Rule};
use rubric_rules::style::documentation::Documentation;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/documentation/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Documentation.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/Documentation"));
}

#[test]
fn no_violation_on_clean() {
    let src = "# A well-documented class\nclass Foo\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Documentation.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_inner_class() {
    // Inner class Bar is indented — should not be flagged even without a doc comment
    let src = "# Top-level class\nclass Foo\n  class Bar\n    def hello; end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Documentation.check_source(&ctx);
    // Foo has a doc comment, Bar is inner (indented) — no violations expected
    assert!(
        diags.is_empty(),
        "expected no FP for inner class with parent having doc, got: {:?}",
        diags
    );
}

#[test]
fn no_false_positive_for_class_self() {
    // class << self is always indented — should not be flagged
    let src = "# Doc\nclass Foo\n  class << self\n    def bar; end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Documentation.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no FP for class << self, got: {:?}",
        diags
    );
}

#[test]
fn still_detects_top_level_class_without_doc() {
    let src = "class Foo\n  def hello; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Documentation.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected violation for top-level class without doc, got none"
    );
}
