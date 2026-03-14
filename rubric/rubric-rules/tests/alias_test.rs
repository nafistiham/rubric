use rubric_core::{LintContext, Rule};
use rubric_rules::style::alias::Alias;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/alias/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/alias/clean.rb");

#[test]
fn detects_alias_keyword() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = Alias.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/Alias"));
}

#[test]
fn no_violation_for_alias_method() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = Alias.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn does_not_flag_alias_in_comment() {
    let src = "# alias foo bar\nclass Foo; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Alias.check_source(&ctx);
    assert!(diags.is_empty(), "alias in comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_alias_method() {
    let src = "alias_method :new_name, :old_name\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Alias.check_source(&ctx);
    assert!(diags.is_empty(), "alias_method should not be flagged, got: {:?}", diags);
}

#[test]
fn flags_alias_with_message() {
    let src = "alias new_name old_name\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = Alias.check_source(&ctx);
    assert!(!diags.is_empty(), "alias keyword should be flagged");
    assert!(diags[0].message.contains("alias_method"));
}
