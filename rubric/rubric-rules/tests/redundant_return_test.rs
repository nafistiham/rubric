use rubric_core::{LintContext, Rule};
use rubric_rules::style::redundant_return::RedundantReturn;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/redundant_return/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/redundant_return/corrected.rb");

#[test]
fn detects_redundant_return() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantReturn.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for redundant return, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/RedundantReturn"));
}

#[test]
fn no_violation_without_redundant_return() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = RedundantReturn.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// `return` inside a `case/else` branch is NOT redundant when the method has
// code after the `case` block.  The missing `case` opener caused the case's
// `end` to look like the def's `end`, triggering a false positive.
#[test]
fn no_false_positive_return_inside_case_else() {
    let src = "def action\n  case x\n  when :a\n    redirect_to path_a\n  else\n    return redirect_to path_b\n  end\n  redirect_to fallback\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantReturn.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "`return` inside case/else must not be flagged when method continues after `end`: {:?}",
        diags
    );
}

// `return` as the true last expression of a method (after a case block) is
// still detected correctly.
#[test]
fn detects_return_as_last_expression_after_case() {
    let src = "def foo\n  case x\n  when :a\n    1\n  else\n    2\n  end\n  return result\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantReturn.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "`return result` as last expression after case block should be flagged"
    );
}
