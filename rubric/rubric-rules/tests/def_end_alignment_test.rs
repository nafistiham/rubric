use rubric_core::{LintContext, Rule};
use rubric_rules::layout::def_end_alignment::DefEndAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/def_end_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/DefEndAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "def bar\n  'bar'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_do_block_inside_def() {
    let src = concat!(
        "def foo\n",
        "  case x\n",
        "  when :a\n",
        "    [1,2].each do |i|\n",
        "      puts i\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for do-block inside def with case, got: {:?}", diags);
}

#[test]
fn still_detects_misaligned_def_end() {
    let src = "def foo\n  1\n    end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned def end, got none");
}

#[test]
fn no_violation_for_inline_if_inside_def() {
    // `end` at aligned position closes the inline `if`, not the `def`
    let src = concat!(
        "def email(name: nil, domain: nil)\n",
        "  local_part = if domain\n",
        "                 foo(name: name, domain: domain)\n",
        "               else\n",
        "                 foo(name: name)\n",
        "               end\n",
        "  local_part\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for inline if inside def, got: {:?}", diags);
}

#[test]
fn no_violation_for_multiple_inline_ifs_inside_def() {
    let src = concat!(
        "def foo\n",
        "  a = if x\n",
        "        1\n",
        "      else\n",
        "        2\n",
        "      end\n",
        "  b = unless y\n",
        "        3\n",
        "      else\n",
        "        4\n",
        "      end\n",
        "  a + b\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = DefEndAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for multiple inline ifs, got: {:?}", diags);
}
