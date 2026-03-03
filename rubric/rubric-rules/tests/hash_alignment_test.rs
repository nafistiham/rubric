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

// ── True positive: extra spaces before `=>` (table-style padding) ────────────
#[test]
fn detects_extra_spaces_before_rocket() {
    let src = "{\n  :a   => 1,\n  :bb  => 2,\n  :ccc => 3\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = HashAlignment.check_source(&ctx);
    assert!(!diags.is_empty(), "extra spaces before `=>` not detected");
    assert!(diags.iter().all(|d| d.rule == "Layout/HashAlignment"));
}
