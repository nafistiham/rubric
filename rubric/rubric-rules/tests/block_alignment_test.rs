use rubric_core::{LintContext, Rule};
use rubric_rules::layout::block_alignment::BlockAlignment;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/block_alignment/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Layout/BlockAlignment"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo do\n  bar\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

#[test]
fn no_false_positive_for_do_block_inside_case() {
    // do-block nested inside case (non-do inner construct)
    let src = concat!(
        "foo.each do\n",
        "  case x\n",
        "  when :a\n",
        "    [1,2].each do |i|\n",
        "      puts i\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for do-block inside case, got: {:?}", diags);
}

#[test]
fn no_false_positive_for_do_block_inside_if() {
    let src = concat!(
        "foo.each do\n",
        "  if cond\n",
        "    bar.each do |x|\n",
        "      puts x\n",
        "    end\n",
        "  end\n",
        "end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no FP for do-block inside if, got: {:?}", diags);
}

#[test]
fn still_detects_misaligned_block_end() {
    let src = "foo.each do\n  bar\n    end\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = BlockAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violation for misaligned block end, got none");
}
