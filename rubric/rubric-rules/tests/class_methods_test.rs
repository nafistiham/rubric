use rubric_core::{LintContext, Rule};
use rubric_rules::style::class_methods::ClassMethods;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/class_methods/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassMethods.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

#[test]
fn no_violation_on_clean() {
    let src = "class Foo\n  class << self\n    def bar\n      'bar'\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// RuboCop's Style/ClassMethods only flags def self.method inside a module body.
// def self.method inside a class body is perfectly idiomatic and must NOT be flagged.
#[test]
fn no_false_positive_for_class_method_in_class() {
    let src = "class DeviseController < ApplicationController\n  def self.internal_methods\n    super << :_prefixes\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside a class body must NOT be flagged, but got: {:?}",
        diags
    );
}

// def self.method inside a module body must be flagged.
#[test]
fn still_detects_class_method_in_module() {
    let src = "module Foo\n  def self.bar\n    'bar'\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "def self.method inside a module body must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Style/ClassMethods"));
}

// def self.method at the top level (no class/module context) must NOT be flagged.
#[test]
fn no_false_positive_for_top_level_class_method() {
    let src = "def self.top_level\n  42\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method at top level must NOT be flagged, but got: {:?}",
        diags
    );
}

// Nested: def self.method inside a class that is nested inside a module must NOT flag.
#[test]
fn no_false_positive_for_class_nested_in_module() {
    let src = "module MyMod\n  class MyClass\n    def self.helper\n      true\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside a class (even nested in a module) must NOT be flagged, but got: {:?}",
        diags
    );
}
