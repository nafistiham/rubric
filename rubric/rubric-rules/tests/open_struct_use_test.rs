use rubric_core::{LintContext, Rule};
use rubric_rules::style::open_struct_use::OpenStructUse;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/open_struct_use/offending.rb");
const CLEAN: &str = include_str!("fixtures/style/open_struct_use/clean.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/OpenStructUse"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), CLEAN);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_open_struct_new() {
    let src = "person = OpenStruct.new(name: 'Alice')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(!diags.is_empty(), "OpenStruct.new should be flagged");
    assert_eq!(diags[0].rule, "Style/OpenStructUse");
    assert!(diags[0].message.contains("OpenStruct"));
}

#[test]
fn detects_require_ostruct_single_quotes() {
    let src = "require 'ostruct'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(!diags.is_empty(), "require 'ostruct' should be flagged");
}

#[test]
fn detects_require_ostruct_double_quotes() {
    let src = "require \"ostruct\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(!diags.is_empty(), "require \"ostruct\" should be flagged");
}

#[test]
fn does_not_flag_struct_new() {
    let src = "Person = Struct.new(:name, :age)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(diags.is_empty(), "Struct.new should not be flagged");
}

#[test]
fn does_not_flag_comment() {
    let src = "# person = OpenStruct.new(name: 'Alice')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_require_other_lib() {
    let src = "require 'json'\nrequire 'set'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OpenStructUse.check_source(&ctx);
    assert!(diags.is_empty(), "other requires should not be flagged");
}
