use rubric_core::{LintContext, Rule};
use rubric_rules::style::negated_if::NegatedIf;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/negated_if/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/negated_if/corrected.rb");

#[test]
fn detects_negated_if() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for `if !`, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedIf"));
}

#[test]
fn no_violation_with_unless() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = NegatedIf.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Modifier-form `if !` must be detected ────────────────────────────────────
// `do_something if !condition` — `if` is in modifier position after an
// expression. The current check only fires when `if` is the first token.
#[test]
fn detects_modifier_negated_if() {
    let src = "def foo\n  do_something if !condition\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "modifier `if !` should be flagged");
    assert!(diags.iter().all(|d| d.rule == "Style/NegatedIf"));
}

#[test]
fn detects_return_modifier_negated_if() {
    let src = "def bar\n  return x if !flag\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "modifier `if !` after return should be flagged");
}

// ── Block-form `if !` still fires ────────────────────────────────────────────
#[test]
fn still_detects_block_negated_if() {
    let src = "if !valid?\n  puts 'bad'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "block-form `if !` should still be flagged");
}

// ── Compound conditions must NOT be flagged ───────────────────────────────────
// RuboCop's NegatedIf only flags a SINGLE negated condition. When `!` is
// combined with `&&` or `||`, the compound expression cannot be simply negated
// with `unless`, so RuboCop leaves it alone.
#[test]
fn no_violation_for_block_compound_and_condition() {
    // `if !done && count > 0` — compound: first operand negated but joined with &&
    let src = "if !done && count > 0\n  work\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "block-form `if !x && y` should not be flagged (compound condition), got: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_block_compound_or_condition() {
    // `if !capsules || other` — compound with ||
    let src = "if !capsules || fallback\n  use_fallback\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "block-form `if !x || y` should not be flagged (compound condition), got: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_modifier_compound_and_condition() {
    // `expr if !global && !other` — modifier form with compound condition
    let src = "location = root + location if !global && !location.start_with?(root)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "modifier `if !x && y` should not be flagged (compound condition), got: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_modifier_compound_or_condition() {
    // `return if !ids || ids.empty?` — modifier form with || compound
    let src = "return if !ids || ids.empty?\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "modifier `if !x || y` should not be flagged (compound condition), got: {:?}",
        diags
    );
}

// ── Single negation is still flagged after compound exemption ─────────────────
#[test]
fn still_detects_single_block_negated_if_after_compound_fix() {
    // `if !job.has_key?(key)` — single negation, still a violation
    let src = "if !job.has_key?(key)\n  fill_in_default\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(!diags.is_empty(), "block-form `if !single_cond` should still be flagged");
}

#[test]
fn no_violation_for_block_multiline_compound_trailing_or() {
    // `if !File.exist?(path) ||` — trailing `||` means condition continues on
    // the next line. This is compound and must not be flagged.
    let src = "if !File.exist?(path) ||\n    (File.directory?(path))\n  warn 'bad'\nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = NegatedIf.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "block-form `if !x ||` (trailing operator, multiline compound) should not be flagged, got: {:?}",
        diags
    );
}
