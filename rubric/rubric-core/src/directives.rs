/// Support for `# rubocop:disable` / `# rubocop:enable` inline directives.
///
/// Three forms are recognised:
///
/// **Form 1 — inline (suppresses only that line):**
/// ```ruby
/// foo and bar # rubocop:disable Style/AndOr
/// ```
///
/// **Form 2 — block (suppresses until matching enable or EOF):**
/// ```ruby
/// # rubocop:disable Style/AndOr
/// foo and bar
/// # rubocop:enable Style/AndOr
/// ```
///
/// **Form 3 — all cops:**
/// ```ruby
/// # rubocop:disable all
/// # rubocop:enable all
/// ```
use std::collections::HashMap;

/// Parse the source and return a map of `cop_name → [(start_line, end_line)]`
/// where both line numbers are 0-indexed and the range is inclusive on both ends.
///
/// The special key `"all"` means every cop is disabled in that range.
pub fn parse_disabled_regions(source: &str) -> HashMap<String, Vec<(usize, usize)>> {
    // cop_name → list of open start lines (not yet matched with an enable)
    let mut open_blocks: HashMap<String, Vec<usize>> = HashMap::new();
    // cop_name → completed (start, end) ranges
    let mut regions: HashMap<String, Vec<(usize, usize)>> = HashMap::new();

    for (line_idx, raw_line) in source.lines().enumerate() {
        let trimmed = raw_line.trim();

        // Detect `# rubocop:disable …` or `# rubocop:enable …`
        // They can appear:
        //   a) as a standalone comment line  → block directive
        //   b) after code on the same line   → inline directive (disable only)
        if let Some(directive_str) = extract_rubocop_directive(trimmed) {
            if let Some(cops_str) = directive_str.strip_prefix("disable") {
                let cops_str = cops_str.trim();
                let cops = parse_cop_list(cops_str);

                // Determine if this is inline (code precedes the comment) or block.
                // A standalone comment line starts with `#` (after trimming).
                let is_block = trimmed.starts_with('#');

                if is_block {
                    // Block disable: record open start
                    for cop in cops {
                        open_blocks.entry(cop).or_default().push(line_idx);
                    }
                } else {
                    // Inline disable: single-line suppression
                    for cop in cops {
                        regions.entry(cop).or_default().push((line_idx, line_idx));
                    }
                }
            } else if let Some(cops_str) = directive_str.strip_prefix("enable") {
                let cops_str = cops_str.trim();
                let cops = parse_cop_list(cops_str);

                for cop in cops {
                    if let Some(starts) = open_blocks.get_mut(&cop) {
                        if let Some(start) = starts.pop() {
                            regions.entry(cop).or_default().push((start, line_idx));
                        }
                    }
                }
            }
        }
    }

    // Any still-open blocks run until EOF (last line index)
    let last_line = source.lines().count().saturating_sub(1);
    for (cop, starts) in open_blocks {
        for start in starts {
            regions.entry(cop.clone()).or_default().push((start, last_line));
        }
    }

    regions
}

/// Given the raw source and a list of diagnostics, remove any diagnostic whose
/// rule is disabled (via a rubocop directive) on the diagnostic's line.
pub fn filter_disabled_by_directives(
    source: &str,
    diagnostics: Vec<crate::types::Diagnostic>,
    line_start_offsets: &[u32],
) -> Vec<crate::types::Diagnostic> {
    if diagnostics.is_empty() {
        return diagnostics;
    }

    // Quick pre-check: only parse directives if the source actually contains one.
    if !source.contains("rubocop:disable") {
        return diagnostics;
    }

    let regions = parse_disabled_regions(source);
    if regions.is_empty() {
        return diagnostics;
    }

    diagnostics
        .into_iter()
        .filter(|diag| {
            // Determine which 0-based line the diagnostic falls on.
            let line_idx = line_for_offset(diag.range.start, line_start_offsets);

            // Check "all" first
            if is_suppressed_on_line(&regions, "all", line_idx) {
                return false; // suppressed
            }

            // Check the specific cop.  Rubocop directive names may omit the
            // category prefix (e.g. "LineLength" instead of "Layout/LineLength").
            // We accept both the full name and the bare name after the last `/`.
            let full_name = diag.rule;
            let bare_name = full_name.rsplit('/').next().unwrap_or(full_name);

            if is_suppressed_on_line(&regions, full_name, line_idx) {
                return false;
            }
            if is_suppressed_on_line(&regions, bare_name, line_idx) {
                return false;
            }

            true // keep
        })
        .collect()
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Return the 0-based line index that contains `offset`.
fn line_for_offset(offset: u32, line_start_offsets: &[u32]) -> usize {
    if line_start_offsets.is_empty() {
        return 0;
    }
    line_start_offsets
        .partition_point(|&start| start <= offset)
        .saturating_sub(1)
}

/// Return `true` if `cop` has at least one disabled region covering `line`.
fn is_suppressed_on_line(
    regions: &HashMap<String, Vec<(usize, usize)>>,
    cop: &str,
    line: usize,
) -> bool {
    regions
        .get(cop)
        .map(|ranges| ranges.iter().any(|&(s, e)| line >= s && line <= e))
        .unwrap_or(false)
}

/// Extract the body of a `rubocop:disable …` or `rubocop:enable …` directive
/// from a trimmed line.  Returns `Some("disable Style/AndOr")` or
/// `Some("enable Style/AndOr")` etc., or `None` if no directive present.
fn extract_rubocop_directive(trimmed: &str) -> Option<&str> {
    // Find "rubocop:" anywhere in the line (can be after code + spaces + #)
    let marker = "rubocop:";
    let pos = trimmed.find(marker)?;
    let after = &trimmed[pos + marker.len()..];
    // after should start with "disable" or "enable"
    if after.starts_with("disable") || after.starts_with("enable") {
        Some(after)
    } else {
        None
    }
}

/// Parse a comma-separated list of cop names, e.g.
/// `"Style/AndOr, Layout/LineLength"` → `["Style/AndOr", "Layout/LineLength"]`.
/// The special token `"all"` is kept as-is.
fn parse_cop_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Diagnostic, Severity, TextRange};

    fn make_diag(rule: &'static str, start: u32) -> Diagnostic {
        Diagnostic {
            rule,
            message: "test".to_string(),
            range: TextRange::new(start, start + 1),
            severity: Severity::Warning,
        }
    }

    // ── parse_disabled_regions ────────────────────────────────────────────

    #[test]
    fn inline_disable_produces_single_line_region() {
        let src = "foo and bar # rubocop:disable Style/AndOr\n";
        let regions = parse_disabled_regions(src);
        assert_eq!(regions.get("Style/AndOr"), Some(&vec![(0, 0)]));
    }

    #[test]
    fn block_disable_without_enable_runs_to_eof() {
        let src = "# rubocop:disable Style/AndOr\nfoo and bar\n";
        let regions = parse_disabled_regions(src);
        // 2 lines (0-indexed: 0, 1) — opened on line 0, no enable → end = 1
        assert_eq!(regions.get("Style/AndOr"), Some(&vec![(0, 1)]));
    }

    #[test]
    fn block_disable_with_enable_closes_region() {
        let src = "# rubocop:disable Style/AndOr\nfoo\n# rubocop:enable Style/AndOr\nbaz\n";
        let regions = parse_disabled_regions(src);
        // Disabled on lines 0–2 (inclusive)
        assert_eq!(regions.get("Style/AndOr"), Some(&vec![(0, 2)]));
    }

    #[test]
    fn multiple_cops_in_one_directive() {
        let src = "# rubocop:disable Style/AndOr, Layout/LineLength\n";
        let regions = parse_disabled_regions(src);
        assert!(regions.contains_key("Style/AndOr"));
        assert!(regions.contains_key("Layout/LineLength"));
    }

    #[test]
    fn disable_all_keyword() {
        let src = "# rubocop:disable all\nfoo\n# rubocop:enable all\n";
        let regions = parse_disabled_regions(src);
        assert_eq!(regions.get("all"), Some(&vec![(0, 2)]));
    }

    #[test]
    fn no_directives_returns_empty_map() {
        let src = "x = 1\ny = 2\n";
        let regions = parse_disabled_regions(src);
        assert!(regions.is_empty());
    }

    // ── filter_disabled_by_directives ─────────────────────────────────────

    #[test]
    fn inline_disable_removes_diagnostic_on_that_line() {
        // "foo and bar # rubocop:disable Style/AndOr\n" — 42 chars
        let src = "foo and bar # rubocop:disable Style/AndOr\n";
        // offset 0 is on line 0
        let offsets = vec![0u32];
        let diags = vec![make_diag("Style/AndOr", 0)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert!(result.is_empty(), "diagnostic should be filtered out");
    }

    #[test]
    fn inline_disable_keeps_diagnostic_on_other_line() {
        let src = "ok_line\nfoo and bar # rubocop:disable Style/AndOr\n";
        // line 0 starts at 0, line 1 starts at 8
        let offsets = vec![0u32, 8u32];
        // Diagnostic is on line 0, which is NOT disabled
        let diags = vec![make_diag("Style/AndOr", 0)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert_eq!(result.len(), 1, "diagnostic on non-disabled line should remain");
    }

    #[test]
    fn block_disable_removes_diagnostics_in_range() {
        let src = "# rubocop:disable Style/AndOr\nfoo and bar\nbaz or qux\n# rubocop:enable Style/AndOr\n";
        // line 0: "# rubocop:disable Style/AndOr\n" = 31 chars
        // line 1: "foo and bar\n" = 12 chars, starts at 31
        // line 2: "baz or qux\n" = 11 chars, starts at 43
        // line 3: "# rubocop:enable Style/AndOr\n" starts at 54
        let offsets = vec![0u32, 31u32, 43u32, 54u32];
        let diags = vec![
            make_diag("Style/AndOr", 31), // line 1 — disabled
            make_diag("Style/AndOr", 43), // line 2 — disabled
            make_diag("Style/AndOr", 54), // line 3 — enable line, still disabled (inclusive)
        ];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert!(result.is_empty(), "all three should be suppressed");
    }

    #[test]
    fn disable_all_suppresses_any_cop() {
        let src = "# rubocop:disable all\nany_cop_line\n# rubocop:enable all\n";
        let offsets = vec![0u32, 22u32, 36u32];
        // A diagnostic for some unrelated cop on line 1
        let diags = vec![make_diag("Layout/LineLength", 22)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert!(result.is_empty(), "'all' should suppress any cop");
    }

    #[test]
    fn bare_cop_name_matches_full_name() {
        // Users sometimes write `# rubocop:disable LineLength` (no category)
        let src = "x = 1 # rubocop:disable LineLength\n";
        let offsets = vec![0u32];
        let diags = vec![make_diag("Layout/LineLength", 0)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert!(result.is_empty(), "bare name should match full cop name");
    }

    #[test]
    fn no_directive_in_source_returns_all_diagnostics() {
        let src = "x = 1\ny = 2\n";
        let offsets = vec![0u32, 6u32];
        let diags = vec![make_diag("Layout/LineLength", 0)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn different_cop_not_suppressed_by_disable() {
        let src = "x = 1 # rubocop:disable Style/AndOr\n";
        let offsets = vec![0u32];
        // A different cop — should not be suppressed
        let diags = vec![make_diag("Layout/LineLength", 0)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert_eq!(result.len(), 1, "unrelated cop should not be filtered");
    }

    #[test]
    fn diagnostic_after_enabled_region_is_not_suppressed() {
        let src = "# rubocop:disable Style/AndOr\nfoo\n# rubocop:enable Style/AndOr\nbar and baz\n";
        // line 0 offset 0, line 1 offset 31, line 2 offset 35, line 3 offset 65
        let offsets = vec![0u32, 31u32, 35u32, 65u32];
        // Diagnostic on line 3 — after the enable → should NOT be filtered
        let diags = vec![make_diag("Style/AndOr", 65)];
        let result = filter_disabled_by_directives(src, diags, &offsets);
        assert_eq!(result.len(), 1, "diagnostic after enable should remain");
    }
}
