use crate::types::{Fix, TextEdit};

/// Apply a set of fixes to `source`, returning the corrected string.
///
/// Edits are applied in descending start-offset order so earlier edits
/// don't shift the offsets of later ones.
///
/// # Panics
/// Panics if two edits have overlapping ranges (indicates a bug in the rule).
pub fn apply_fixes(source: &str, fixes: &[Fix]) -> String {
    // Flatten all edits from all fixes
    let mut edits: Vec<TextEdit> = fixes
        .iter()
        .flat_map(|f| f.edits.iter().cloned())
        .collect();

    if edits.is_empty() {
        return source.to_string();
    }

    // Sort descending by start offset so we apply from end to start
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));

    // Assert no overlapping edits (debug guard — indicates a rule bug)
    for i in 0..edits.len().saturating_sub(1) {
        assert!(
            edits[i].range.start >= edits[i + 1].range.end,
            "overlapping edits: [{}, {}) and [{}, {})",
            edits[i + 1].range.start,
            edits[i + 1].range.end,
            edits[i].range.start,
            edits[i].range.end,
        );
    }

    let mut result = source.to_string();
    for edit in &edits {
        let start = edit.range.start as usize;
        let end = edit.range.end as usize;
        result.replace_range(start..end, &edit.replacement);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Fix, FixSafety, TextEdit, TextRange};

    fn make_fix(start: u32, end: u32, replacement: &str) -> Fix {
        Fix {
            edits: vec![TextEdit {
                range: TextRange::new(start, end),
                replacement: replacement.to_string(),
            }],
            safety: FixSafety::Safe,
        }
    }

    #[test]
    fn empty_fixes_returns_source_unchanged() {
        let source = "hello\n";
        let result = apply_fixes(source, &[]);
        assert_eq!(result, source);
    }

    #[test]
    fn single_edit_replaces_range() {
        let source = "hello world\n";
        // Replace "world" (bytes 6..11) with "Rust"
        let fix = make_fix(6, 11, "Rust");
        let result = apply_fixes(source, &[fix]);
        assert_eq!(result, "hello Rust\n");
    }

    #[test]
    fn multiple_non_overlapping_edits_applied_correctly() {
        let source = "abc def ghi\n";
        // Replace "abc" (0..3) with "ABC" and "ghi" (8..11) with "GHI"
        let fix1 = make_fix(0, 3, "ABC");
        let fix2 = make_fix(8, 11, "GHI");
        let result = apply_fixes(source, &[fix1, fix2]);
        assert_eq!(result, "ABC def GHI\n");
    }

    #[test]
    fn deletion_edit_removes_range() {
        let source = "hello   world\n";
        // Remove extra spaces (bytes 5..8 → "")
        let fix = make_fix(5, 8, "");
        let result = apply_fixes(source, &[fix]);
        assert_eq!(result, "helloworld\n");
    }

    #[test]
    fn insertion_edit_with_zero_length_range() {
        let source = "helloworld\n";
        // Insert " " at position 5 (start == end == 5)
        let fix = make_fix(5, 5, " ");
        let result = apply_fixes(source, &[fix]);
        assert_eq!(result, "hello world\n");
    }

    #[test]
    fn edits_applied_regardless_of_input_order() {
        let source = "abc def\n";
        // Provide in ascending order — function must sort descending before applying
        let fix1 = make_fix(0, 3, "ABC");
        let fix2 = make_fix(4, 7, "DEF");
        // Even though fix1 comes before fix2 in the vec, both must be applied correctly
        let result = apply_fixes(source, &[fix1, fix2]);
        assert_eq!(result, "ABC DEF\n");
    }

    #[test]
    fn fix_with_multiple_edits() {
        // A single Fix can have multiple TextEdits
        let source = "aXbYc\n";
        let fix = Fix {
            edits: vec![
                TextEdit { range: TextRange::new(1, 2), replacement: "".to_string() }, // remove X
                TextEdit { range: TextRange::new(3, 4), replacement: "".to_string() }, // remove Y
            ],
            safety: FixSafety::Safe,
        };
        let result = apply_fixes(source, &[fix]);
        assert_eq!(result, "abc\n");
    }
}
