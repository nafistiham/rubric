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

// ── False positive: `||= if` compound assignment followed by if ──────────────
#[test]
fn no_false_positive_for_or_assign_inline_if() {
    let src = concat!(
        "def display_args\n",
        "  @cache ||= if cond1\n",
        "    if cond2\n",
        "      val\n",
        "    end\n",
        "    args\n",
        "  else\n",
        "    args\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "||= if falsely flagged: {:?}", diags);
}

// ── False positive: `||= ... || begin` inline begin with nested if ────────────
#[test]
fn no_false_positive_for_inline_or_begin_with_nested_if() {
    let src = concat!(
        "def display_class\n",
        "  @klass ||= self['x'] || begin\n",
        "    if cond1\n",
        "      if cond2\n",
        "        args[0]\n",
        "      else\n",
        "        val\n",
        "      end\n",
        "    else\n",
        "      klass\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "|| begin with nested if falsely flagged: {:?}", diags);
}

// ── False positive: inline `if` + nested inline `begin` assignment ────────────
// Pattern: val = if cond / job = begin ... rescue ... end / ... / else / end
#[test]
fn no_false_positive_for_inline_if_with_nested_inline_begin() {
    let src = concat!(
        "def fetch\n",
        "  result = if entry\n",
        "    job = begin\n",
        "      load(entry)\n",
        "    rescue\n",
        "      {}\n",
        "    end\n",
        "    compute(job)\n",
        "  else\n",
        "    0.0\n",
        "  end\n",
        "  result\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = EndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "nested inline begin in inline if falsely flagged: {:?}", diags);
}
