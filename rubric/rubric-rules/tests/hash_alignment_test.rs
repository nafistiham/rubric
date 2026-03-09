use rubric_core::{LintContext, Rule};
use rubric_rules::layout::hash_alignment::HashAlignment;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/hash_alignment/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/hash_alignment/corrected.rb");

#[test]
fn detects_misaligned_hash_rockets() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = HashAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations for misaligned rockets, got none");
    assert!(diags.iter().all(|d| d.rule == "Layout/HashAlignment"));
}

#[test]
fn no_violation_with_aligned_hash_rockets() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

#[test]
fn no_violation_for_rocket_in_comment() {
    let src = "# foo => bar\n# baz => qux\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "comment lines with `=>` must not be flagged, got: {:?}", diags);
}

#[test]
fn no_violation_for_rocket_in_string() {
    let src = "x = \"foo => bar\"\ny = \"baz => qux\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "string literals with `=>` must not be flagged, got: {:?}", diags);
}

// ── False positive: key style (one space before each `=>`) ───────────────────
#[test]
fn no_false_positive_for_key_style_different_lengths() {
    let src = "{\n  \"hostname\" => hostname,\n  \"started_at\" => Time.now,\n  \"pid\" => pid,\n  \"tag\" => tag,\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "key style with different key lengths falsely flagged: {:?}", diags);
}

// ── False positive: method calls with hash args at same indentation ───────────
#[test]
fn no_false_positive_for_method_call_hash_args() {
    let src = "  get \"job\" => \"job#index\"\n  get \"job/email\" => \"job#email\"\n  get \"job/long\" => \"job#long\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "separate method calls with hash args falsely flagged: {:?}", diags);
}

// ── False positive: multi-line method call continuation ──────────────────────
#[test]
fn no_false_positive_for_multiline_method_call_continuation() {
    let src = "      Process.new(hash.merge(\"busy\" => busy.to_i,\n        \"beat\" => beat.to_f,\n        \"quiet\" => quiet,\n      ))\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(diags.is_empty(), "multi-line method call hash continuation falsely flagged: {:?}", diags);
}

// ── Table-aligned groups are now allowed (table style) ───────────────────────
// If all rockets in a hash are at the same column, it's consistent table style.
// RuboCop with `EnforcedStyle: table` allows this — we match that behavior to
// avoid 371 FPs in the jekyll benchmark.
#[test]
fn no_false_positive_for_table_aligned_hash() {
    // All three rockets are at the same column — table style, must not be flagged.
    let src = "{\n  :a   => 1,\n  :bb  => 2,\n  :ccc => 3\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "consistently aligned rockets must not be flagged; got: {:?}",
        diags
    );
}

// ── True positive: inconsistently aligned rockets ────────────────────────────
// When rockets are at different columns in the same hash, it's neither valid
// `key` style (1 space each) nor valid `table` style (same column) — flag it.
#[test]
fn detects_inconsistently_aligned_rockets() {
    // "short"     => at column 12, "much_longer_key" => at column 19 — inconsistent
    let src = "{\n  \"short\"     => \"val\",\n  \"much_longer_key\" => \"val2\"\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "inconsistently aligned rockets must be flagged"
    );
    assert!(diags.iter().all(|d| d.rule == "Layout/HashAlignment"));
}

// ── True positive: lone rocket with extra spaces (no aligned group) ───────────
#[test]
fn detects_lone_extra_spaced_rocket() {
    // A single `=>` with extra spaces, no neighboring rockets at same column.
    let src = "result = {  \"key\"   => value }\nother_result = {  \"key\" => value }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "lone extra-spaced rocket with no aligned group must be flagged"
    );
}
