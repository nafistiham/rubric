use rubric_core::{LintContext, Rule};
use rubric_rules::lint::float_out_of_range::FloatOutOfRange;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/lint/float_out_of_range/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/lint/float_out_of_range/corrected.rb");

#[test]
fn detects_float_out_of_range() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(!diags.is_empty(), "expected violations, got none");
    assert!(diags.iter().all(|d| d.rule == "Lint/FloatOutOfRange"));
}

#[test]
fn no_violation_for_normal_float() {
    let ctx = LintContext::new(Path::new("test.rb"), CORRECTED);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
}

// --- False-positive regression tests ---

#[test]
fn no_violation_for_hex_string_with_e_sequences() {
    // Hex strings in string literals that contain `eNNN` patterns should not be flagged.
    let src = r#"secret = "35c5108120cb479eecb4e947e423cad6da6f38327cf0ebb323e30816d74fa01f""#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "hex string in double-quoted string should not be flagged: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_single_quoted_hex_string() {
    let src =
        r#"hash = '973dfe463ec85785f5f95af5ba3906eedb2d931c24e69824a89ea65dba4e813b'"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "hex string in single-quoted string should not be flagged: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_json_string_with_timestamps() {
    // JSON serialized strings with numeric timestamps like `1568305542339` should be fine.
    let src = r#"JOB = '{"created_at":1568305542339,"enqueued_at":1568305542370}'"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "JSON string with large integers should not be flagged: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_e_inside_identifier_context() {
    // `e` that is followed by letters is not scientific notation.
    let src = "name = idx_on_account_e947_event_id";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "identifier containing e+digits should not be flagged: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_db_index_name_string() {
    // DB migration index names like "idx_on_year_account_id_ff3e167cef" contain `e167`.
    let src = r#"t.index ["year", "account_id"], name: "idx_on_year_account_id_ff3e167cef", unique: true"#;
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "index name strings should not be flagged: {:?}",
        diags
    );
}

#[test]
fn no_violation_for_number_in_comment() {
    let src = "# x = 1e999 would overflow";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "floats in comments should not be flagged: {:?}",
        diags
    );
}

#[test]
fn detects_actual_overflow_literal() {
    // 1e999 genuinely overflows to Infinity in Ruby.
    let src = "x = 1e999";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "1e999 should be flagged");
}

#[test]
fn detects_actual_underflow_literal() {
    // 1e-999 underflows to 0.0 in Ruby.
    let src = "x = 1e-999";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert_eq!(diags.len(), 1, "1e-999 should be flagged");
}

#[test]
fn no_violation_for_normal_scientific_notation() {
    // 1.5e10 is perfectly representable.
    let src = "x = 1.5e10";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(diags.is_empty(), "1.5e10 should not be flagged: {:?}", diags);
}

#[test]
fn no_violation_for_float_infinity_constant() {
    // Float::INFINITY is a constant reference, not a literal.
    let src = "x = Float::INFINITY\ny = -Float::INFINITY\nz = Float::NAN";
    let ctx = LintContext::new(Path::new("test.rb"), src);
    let diags = FloatOutOfRange.check_source(&ctx);
    assert!(
        diags.is_empty(),
        "Float::INFINITY should not be flagged: {:?}",
        diags
    );
}
