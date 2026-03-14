use rubric_core::{LintContext, Rule};
use rubric_rules::style::trivial_accessors::TrivialAccessors;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/trivial_accessors/offending.rb");
const CLEAN: &str =
    include_str!("fixtures/style/trivial_accessors/clean.rb");

#[test]
fn detects_trivial_accessors() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/TrivialAccessors"));
}

#[test]
fn no_violation_for_clean_accessors() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn flags_trivial_reader_multiline() {
    let src = "def name\n  @name\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(!diags.is_empty(), "trivial reader should be flagged");
    assert!(diags[0].message.contains("attr_reader"));
}

#[test]
fn flags_trivial_writer_multiline() {
    let src = "def age=(val)\n  @age = val\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(!diags.is_empty(), "trivial writer should be flagged");
    assert!(diags[0].message.contains("attr_writer"));
}

#[test]
fn flags_trivial_reader_one_liner() {
    let src = "def title; @title; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(!diags.is_empty(), "trivial reader one-liner should be flagged");
    assert!(diags[0].message.contains("attr_reader"));
}

#[test]
fn flags_trivial_writer_one_liner() {
    let src = "def title=(v); @title = v; end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(!diags.is_empty(), "trivial writer one-liner should be flagged");
    assert!(diags[0].message.contains("attr_writer"));
}

#[test]
fn does_not_flag_complex_getter() {
    let src = "def name\n  @name.upcase\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(diags.is_empty(), "complex getter should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_getter_with_default() {
    let src = "def data\n  @data ||= compute\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(diags.is_empty(), "getter with default should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_comment_line() {
    let src = "# def name\n#   @name\n# end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = TrivialAccessors.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines should not be flagged, got: {:?}", diags);
}
