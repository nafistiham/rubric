use rubric_core::{LintContext, Rule};
use rubric_rules::layout::rescue_ensure_alignment::RescueEnsureAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/rescue_ensure_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/rescue_ensure_alignment/corrected.rb");

#[test]
fn detects_misaligned_rescue() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/RescueEnsureAlignment"));
}

#[test]
fn no_violation_for_aligned_rescue() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// `rescue` inside a `do...end` block must align with the `do` line's indent,
// not with the enclosing `def`
#[test]
fn no_false_positive_for_rescue_inside_do_block() {
    let src = concat!(
        "  def fields\n",
        "    (self[:fields] || []).filter_map do |f|\n",
        "      Account::Field.new(self, f)\n",
        "    rescue\n",
        "      nil\n",
        "    end\n",
        "  end\n",
    );
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RescueEnsureAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "rescue inside do block falsely flagged: {:?}", diags);
}
