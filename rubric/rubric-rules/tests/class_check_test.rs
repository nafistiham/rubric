use rubric_core::{LintContext, Rule};
use std::path::Path;

#[path = "../src/style/class_check.rs"]
mod class_check;
use class_check::ClassCheck;

const OFFENDING: &str = include_str!("fixtures/style/class_check/offending.rb");
const PASSING: &str = include_str!("fixtures/style/class_check/passing.rb");

#[test]
fn detects_kind_of_and_instance_of() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassCheck.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassCheck"));
}

#[test]
fn no_violation_on_passing() {
    let ctx = LintContext::new(Path::new("test.rb"), PASSING);
    let diags = ClassCheck.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn detects_kind_of_with_correct_message() {
    let src = "x.kind_of?(String)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassCheck.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for kind_of?");
    assert!(
        diags[0].message.contains("kind_of?"),
        "message should mention kind_of?, got: {}",
        diags[0].message
    );
}

#[test]
fn detects_instance_of_with_correct_message() {
    let src = "x.instance_of?(Integer)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassCheck.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for instance_of?");
    assert!(
        diags[0].message.contains("instance_of?"),
        "message should mention instance_of?, got: {}",
        diags[0].message
    );
}

#[test]
fn does_not_flag_comment_lines() {
    let src = "# x.kind_of?(String) is discouraged\nx.is_a?(String)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassCheck.check_source(&ctx);
    assert!(diags.is_empty(), "comment should not be flagged, got: {:?}", diags);
}

#[test]
fn does_not_flag_inside_string() {
    let src = "msg = \"use .kind_of? carefully\"\nother = 'instance_of? is also bad'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassCheck.check_source(&ctx);
    assert!(diags.is_empty(), "string contents should not be flagged, got: {:?}", diags);
}
