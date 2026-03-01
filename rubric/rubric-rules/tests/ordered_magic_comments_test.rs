use rubric_core::{LintContext, Rule};
use rubric_rules::lint::ordered_magic_comments::OrderedMagicComments;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/ordered_magic_comments/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = OrderedMagicComments.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/OrderedMagicComments"));
}

#[test]
fn no_violation_on_clean() {
    let src = "# encoding: utf-8\n# frozen_string_literal: true\n\nx = 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = OrderedMagicComments.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}
