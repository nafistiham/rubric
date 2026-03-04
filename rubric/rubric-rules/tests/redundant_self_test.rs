use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_self::RedundantSelf;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/redundant_self/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/redundant_self/corrected.rb");

#[test]
fn detects_redundant_self() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantSelf"));
}

#[test]
fn no_violation_without_redundant_self() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// `def self.method_name` declaration must NOT be flagged — `self.` is required there.
#[test]
fn no_false_positive_on_def_self_declaration() {
    let source = "module Foo\n  def self.bar\n    42\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`def self.bar` declaration must not be flagged, got: {:?}",
        diags
    );
}

// `self.xxx` calls inside a `def self.method` body must NOT be flagged.
#[test]
fn no_false_positive_self_call_inside_class_method_body() {
    let source = "module Foo\n  def self.bar\n    self.baz\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`self.baz` inside `def self.bar` body must not be flagged, got: {:?}",
        diags
    );
}

// Plain instance methods still generate a violation when `self.method` is used redundantly.
#[test]
fn still_detects_redundant_self_in_instance_method() {
    let source = "def foo\n  self.bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), source);
    let diags = RedundantSelf.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "`self.bar` inside plain `def foo` must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantSelf"));
}
