use rubric_core::{LintContext, Rule};
use rubric_rules::style::class_and_module_children::ClassAndModuleChildren;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/style/class_and_module_children/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Style/ClassAndModuleChildren"));
}

#[test]
fn no_violation_on_clean() {
    let src = "class Foo\n  class Bar\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── False positive: `::` only in parent class reference ──────────────────────
#[test]
fn no_false_positive_for_parent_class_with_namespace() {
    let src = "class ApplicationController < ActionController::Base\nend\nclass ApplicationJob < ActiveJob::Base\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(diags.is_empty(), "parent class with :: falsely flagged: {:?}", diags);
}

// ── True positive: compact notation in the class name itself ──────────────────
#[test]
fn still_detects_compact_notation_in_class_name() {
    let src = "class Foo::Bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(!diags.is_empty(), "compact notation in class name not detected");
}

// ── True positive: compact notation in module name ────────────────────────────
#[test]
fn still_detects_compact_notation_in_module_name() {
    let src = "module Sidekiq::Web\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ClassAndModuleChildren.check_source(&ctx);
    assert!(!diags.is_empty(), "compact notation in module name not detected");
}
