use rubric_core::{LintContext, Rule};
use rubric_rules::layout::end_alignment::EndAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/end_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/end_alignment/corrected.rb");

#[test]
fn detects_misaligned_end() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = EndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/EndAlignment"));
}

#[test]
fn no_violation_for_aligned_end() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_inline_if_assignment() {
    let src = "module Foo\n  class Bar\n    def email\n      local_part = if true\n                     'a'\n                   else\n                     'b'\n                   end\n      local_part\n    end\n  end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for inline if assignment, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_end_dot_method_chain() {
    let src = "def foo\n  [1, 2, 3].map do |x|\n    x + 1\n  end.join(', ')\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for end.method chain, got: {:?}", diags);
}

#[test]
fn still_detects_misaligned_end_after_def() {
    let src = "def foo\n  1\n    end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned end, got none");
}

#[test]
fn no_false_positive_for_shovel_if_inline_conditional() {
    let src = "def foo\n  arr << if cond\n             val1\n           else\n             val2\n           end\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for << if inline conditional, got: {:?}", diags);
}
