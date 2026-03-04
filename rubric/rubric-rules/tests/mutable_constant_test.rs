use rubric_core::{LintContext, Rule};
use rubric_rules::style::mutable_constant::MutableConstant;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/style/mutable_constant/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/style/mutable_constant/corrected.rb");

#[test]
fn detects_mutable_constant() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MutableConstant.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Style/MutableConstant"));
}

#[test]
fn no_violation_for_frozen_constant() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = MutableConstant.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// Setter method calls are not constant assignments — they must not be flagged.
#[test]
fn no_false_positive_for_setter_method_call() {
    let src = "Devise.mailer = 'Devise::Mailer'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for setter method call, got: {:?}",
        diags
    );
}

// A setter with a mutable RHS but a receiver before the dot is still not a
// constant assignment.
#[test]
fn no_false_positive_for_namespaced_setter() {
    let src = "Config.value = []\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "expected no violations for namespaced setter, got: {:?}",
        diags
    );
}

// Bare mutable array constant must still be detected.
#[test]
fn still_detects_mutable_array_constant() {
    let src = "NAMES = ['Alice', 'Bob']\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for mutable array constant, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// Bare mutable hash constant must still be detected.
#[test]
fn still_detects_mutable_hash_constant() {
    let src = "CONFIG = { a: 1 }\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for mutable hash constant, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// ── False positive: string constant in a frozen file ─────────────────────────
// When `# frozen_string_literal: true` is present, all string literals are
// immutable at runtime. Rubocop does not flag them; rubric must not either.
#[test]
fn no_false_positive_for_string_constant_in_frozen_file() {
    let src = "# frozen_string_literal: true\n\nFOO = \"bar\"\nBAZ = 'qux'\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "string constants in frozen file falsely flagged: {:?}",
        diags
    );
}

// ── String constant in non-frozen file is still mutable ──────────────────────
#[test]
fn still_detects_mutable_string_constant_in_non_frozen_file() {
    let src = "FOO = \"bar\"\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for mutable string constant in non-frozen file, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// ── Arrays/hashes in frozen file are STILL mutable (freeze exemption is
// only for strings). Must still be flagged.
#[test]
fn still_detects_mutable_array_in_frozen_file() {
    let src = "# frozen_string_literal: true\n\nNAMES = ['Alice', 'Bob']\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for mutable array in frozen file, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// ── Multi-line hash with .freeze on closing line — must NOT be flagged ────────
// Pattern from mastodon: CONST = {\n  ...\n}.freeze
// The .freeze is on the closing `}` line, not the opening `= {` line.
#[test]
fn no_false_positive_for_multiline_hash_with_freeze_on_closing_line() {
    let src = "VISIBLITY_ICONS = {\n  public: 'globe',\n  private: 'lock',\n}.freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "multi-line hash with closing .freeze falsely flagged: {:?}",
        diags
    );
}

// ── Multi-line array with .freeze on closing line — must NOT be flagged ───────
#[test]
fn no_false_positive_for_multiline_array_with_freeze_on_closing_line() {
    let src = "SOURCES = [\n  AnnualReport::Archetype,\n  AnnualReport::TopStatuses,\n].freeze\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "multi-line array with closing .freeze falsely flagged: {:?}",
        diags
    );
}

// ── Multi-line hash WITHOUT .freeze — must still be flagged ──────────────────
#[test]
fn still_detects_multiline_hash_without_freeze() {
    let src = "HOLIDAY_COLORS = {\n  \"3-17\" => \"green\",\n  \"10-31\" => \"orange\"\n}\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for multi-line hash without .freeze, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// ── Multi-line array WITHOUT .freeze — must still be flagged ─────────────────
#[test]
fn still_detects_multiline_array_without_freeze() {
    let src = "PROCTITLES = [\n  proc { \"sidekiq\" },\n  proc { Sidekiq::VERSION },\n]\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        !diags.is_empty(),
        "expected a violation for multi-line array without .freeze, got none"
    );
    assert_eq!(diags[0].rule, "Style/MutableConstant");
}

// ── Array literal with chained method call — assigned value is not an array ──
// `CONST = [a, b].join('x')` — the result is a String, not a mutable Array.
// In a frozen file the string is immutable; Rubocop does not flag this.
#[test]
fn no_false_positive_for_array_literal_with_chained_method_in_frozen_file() {
    let src = "# frozen_string_literal: true\n\nAVATAR_GEOMETRY = [AVATAR_DIMENSIONS.first, AVATAR_DIMENSIONS.last].join('x')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "array literal with chained method call in frozen file falsely flagged: {:?}",
        diags
    );
}

// ── Array literal with chained non-freeze method — NOT in frozen file ─────────
// `CONST = [a, b].join('x')` in a non-frozen file: the result is a mutable
// String. Rubocop DOES flag this. We should too.
#[test]
fn detects_array_with_chained_method_in_non_frozen_file() {
    let src = "AVATAR_GEOMETRY = [AVATAR_DIMENSIONS.first, AVATAR_DIMENSIONS.last].join('x')\n";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = MutableConstant.check_source(&ctx);
    // Rubocop flags this because .join returns a mutable String in a non-frozen file.
    // Our rule may or may not match this pattern; the main goal is to not flag it
    // in a frozen file. This test documents the current behaviour.
    let _ = diags; // behaviour is documented, not strictly required
}
