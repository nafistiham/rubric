use rubric_core::{LintContext, Rule};
use rubric_rules::layout::extra_spacing::ExtraSpacing;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/extra_spacing/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/extra_spacing/corrected.rb");

#[test]
fn detects_extra_spacing() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/ExtraSpacing"));
}

#[test]
fn no_violation_for_single_spacing() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// ── Alignment spacing after comma must NOT fire ───────────────────────────
// Ruby arrays and hashes commonly use extra spaces after `,` to visually
// align columns: `[10..10,   0..255,   0..255, 1..255]`
#[test]
fn no_false_positive_for_alignment_spacing_after_comma() {
    let src = "RANGES = [\n  [10..10,   0..255,   0..255, 1..255],\n  [127..127, 0..255,   0..255, 1..255],\n]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "alignment spaces after comma should not be flagged: {:?}", diags);
}

// ── Trailing whitespace must NOT fire here (TrailingWhitespace handles it) ─
#[test]
fn no_false_positive_for_trailing_whitespace() {
    let src = "def foo\n  x = 1   \nend\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "trailing whitespace should not be flagged by ExtraSpacing: {:?}", diags);
}

// ── True positive: double space between operator and value still fires ─────
#[test]
fn still_detects_double_space_in_assignment() {
    let src = "x  = 1\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(!diags.is_empty(), "double space before = should still be flagged");
}

// ── Extra spaces before `#` comment (comment-alignment) must NOT fire ─────
#[test]
fn no_false_positive_for_spaces_before_comment() {
    let src = "x = /regex/      # comment\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for comment-alignment, got: {:?}", diags);
}

// ── Column-aligned symbol (`:`) across consecutive lines must NOT fire ──────
// e.g. `after_create_commit  :foo` / `after_destroy_commit :foo` — `:` aligned
#[test]
fn no_false_positive_for_column_aligned_symbol() {
    let src = "  after_create_commit  :increment_counter_caches\n  after_destroy_commit :decrement_counter_caches\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for column-aligned : symbol, got: {:?}", diags);
}

// ── Column-aligned `=` in consecutive assignments must NOT fire ────────────
#[test]
fn no_false_positive_for_column_aligned_assignments() {
    let src = "@query     = foo\n@account   = bar\n@options   = baz\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = ExtraSpacing.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations for column-aligned =, got: {:?}", diags);
}
