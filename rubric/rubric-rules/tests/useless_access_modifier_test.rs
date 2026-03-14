use rubric_core::{LintContext, Rule};
use rubric_rules::lint::useless_access_modifier::UselessAccessModifier;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/useless_access_modifier/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/lint/useless_access_modifier/clean.rb");

#[test]
fn detects_consecutive_access_modifiers() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/UselessAccessModifier"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_redundant_consecutive_modifiers() {
    let src = "class Foo\n  private\n  protected\n  def bar; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "consecutive different modifiers should be flagged");
}

#[test]
fn does_not_flag_modifier_followed_by_def() {
    let src = "class Foo\n  private\n  def secret; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "modifier followed by def should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_private_def() {
    let src = "class Foo\n  private def secret; end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(diags.is_empty(), "private def should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_modifier_at_end_of_class_with_no_following_def() {
    let src = "class Foo\n  def bar; end\n  private\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty(), "trailing modifier before class end should be flagged");
}

#[test]
fn message_contains_modifier_name() {
    let src = "class Foo\n  private\n  private\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = UselessAccessModifier.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(
        diags[0].message.contains("private"),
        "message should contain the modifier name, got: {}",
        diags[0].message
    );
}
