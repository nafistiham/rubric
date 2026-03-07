use rubric_core::{LintContext, Rule};
use rubric_rules::lint::redundant_splat_expansion::RedundantSplatExpansion;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/lint/redundant_splat_expansion/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations in offending.rb");
    assert!(diags.iter().all(|d| d.rule == "Lint/RedundantSplatExpansion"));
}

#[test]
fn no_violation_on_clean() {
    let src = "foo(bar)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations on clean code");
}

// ── False positive: `*[` inside a string literal ─────────────────────────────
// Regex character classes like `]*[` appear inside strings, must not be flagged.
#[test]
fn no_false_positive_for_star_bracket_in_string() {
    let src = "PAT = \"[[:word:]_]*[[:alpha:]]\".freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(diags.is_empty(), "*[ inside string must not be flagged: {:?}", diags);
}

// ── False positive: `*[...].method` — method call on array, not a literal ───
// `*[a, b].compact` is NOT a redundant splat — `.compact` transforms the array.
#[test]
fn no_false_positive_for_splat_with_method_chain() {
    let src = "File.join(*[root_path, prefix].compact)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(diags.is_empty(), "*[...].method must not be flagged: {:?}", diags);
}

// ── False positive: `*[` inside a regex literal ──────────────────────────────
// e.g. `/Li(Po\s|-)[3-4]*[A-Za-z]+/` — the `*` is a quantifier, `[` starts a
// character class, and neither constitutes a splat expansion.
#[test]
fn no_false_positive_for_star_bracket_in_regex() {
    let src = "assert_match(/Li(Po\\s|-)[3-4]*[A-Za-z]+/, @tester.battery_type)\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = RedundantSplatExpansion.check_source(&ctx);
    assert!(diags.is_empty(), "*[ inside regex must not be flagged: {:?}", diags);
}
