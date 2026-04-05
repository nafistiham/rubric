use rubric_core::{LintContext, Rule};
use rubric_rules::lint::missing_super::MissingSuper;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/missing_super/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/lint/missing_super/clean.rb");

#[test]
fn detects_missing_super_in_initialize() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MissingSuper.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/MissingSuper"));
}

#[test]
fn no_violation_when_super_called() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = MissingSuper.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_when_super_with_args() {
    let src = "class Child < Parent\n  def initialize(name)\n    super(name)\n    @name = name\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MissingSuper.check_source(&ctx);
    assert!(diags.is_empty(), "super() should satisfy requirement, got: {:?}", diags);
}

#[test]
fn flags_initialize_without_super_word() {
    let src = "class Foo < Bar\n  def initialize\n    @x = 1\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MissingSuper.check_source(&ctx);
    assert!(!diags.is_empty(), "initialize without super should be flagged");
    assert!(diags[0].message.contains("super"));
}

#[test]
fn does_not_flag_super_in_string() {
    let src = "class Foo < Bar\n  def initialize\n    @msg = \"call super here\"\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MissingSuper.check_source(&ctx);
    // "super" in a string doesn't count — should still flag
    assert!(!diags.is_empty(), "super in string should not count as calling super");
}
