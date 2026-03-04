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

// A class method that uses a trailing `begin..end` block (e.g. `@@x ||= begin ... end`)
// must not cause the scope tracker to desync and flag subsequent def self.methods in the
// same class as violations.
//
// Root cause: `@@day ||= begin` does not start with `begin`, so is_other_block_opener
// returns false and the begin block is not pushed onto the scope stack. The end that
// closes the begin block then pops the scope for def self.day instead, and the end that
// closes def self.day pops the enclosing Class scope. Everything that follows then sees
// Module as the innermost scope and is incorrectly flagged.
#[test]
fn no_false_positive_for_class_method_after_inline_begin() {
    let src = concat!(
        "module Outer\n",
        "  class CLI\n",
        "    def self.day\n",
        "      @@day ||= begin\n",
        "        42\n",
        "      end\n",
        "    end\n",
        "    def self.r\n",
        "      'red'\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method after inline begin..end in a class must NOT be flagged, but got: {:?}",
        diags
    );
}

// def self.method inside an `included do` block must NOT be flagged, even when the
// method body contains an assignment-expression `result = if condition ... end`.
//
// Root cause: `result = if condition` does not start with `if `, so is_other_block_opener
// returned false and the inline-if block was not pushed onto the scope stack. The bare
// `end` that closes the multi-line if expression then pops the scope for the surrounding
// `def self.method` instead, leaving the `included do` scope as innermost for the next
// `def self.` line — which is then falsely flagged as a module-level violation.
#[test]
fn no_false_positive_for_def_self_after_assignment_if_in_included_block() {
    let src = concat!(
        "module User::PamAuthenticable\n",
        "  extend ActiveSupport::Concern\n",
        "\n",
        "  included do\n",
        "    def self.pam_get_user(attributes = {})\n",
        "      resource = if attributes[:email]\n",
        "                   find_by(email: attributes[:email])\n",
        "                 else\n",
        "                   nil\n",
        "                 end\n",
        "      resource\n",
        "    end\n",
        "\n",
        "    def self.authenticate_with_pam(attributes = {})\n",
        "      super\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside an included do block must NOT be flagged even after assignment-if, but got: {:?}",
        diags
    );
}

// Variant: `result = case expr ... end` inside a method body must not desync the scope
// stack and cause a subsequent `def self.` at the same nesting level to be falsely flagged.
#[test]
fn no_false_positive_for_def_self_after_assignment_case_in_included_block() {
    let src = concat!(
        "module Helpers\n",
        "  included do\n",
        "    def handle(val)\n",
        "      result = case val\n",
        "               when :a then 1\n",
        "               when :b then 2\n",
        "               end\n",
        "      result\n",
        "    end\n",
        "\n",
        "    def self.setup!\n",
        "      true\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassMethods.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "def self.method inside an included do block must NOT be flagged after assignment-case, but got: {:?}",
        diags
    );
}
