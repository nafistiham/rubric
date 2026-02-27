use rubric_core::{LintContext, Rule};
use rubric_rules::layout::trailing_whitespace::TrailingWhitespace;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/trailing_whitespace/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/trailing_whitespace/corrected.rb");

#[test]
fn detects_trailing_whitespace() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diagnostics = TrailingWhitespace.check_source(&ctx);

    assert_eq!(diagnostics.len(), 2, "expected 2 violations (lines 1 and 2)");
    assert!(diagnostics.iter().all(|d| d.rule == "Layout/TrailingWhitespace"));
}

#[test]
fn fix_removes_trailing_whitespace() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diagnostics = TrailingWhitespace.check_source(&ctx);

    assert!(!diagnostics.is_empty(), "should have diagnostics to fix");

    // Apply all fixes to source (reverse order to preserve offsets)
    let mut result = OFFENDING.to_string();
    let mut edits: Vec<_> = diagnostics
        .iter()
        .filter_map(|d| TrailingWhitespace.fix(d))
        .flat_map(|f| f.edits)
        .collect();
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    for edit in edits {
        let start = edit.range.start as usize;
        let end   = edit.range.end as usize;
        result.replace_range(start..end, &edit.replacement);
    }

    assert_eq!(result, CORRECTED);
}
