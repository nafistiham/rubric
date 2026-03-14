use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MinMaxComparison;

/// A simple identifier: alphanumeric characters and underscores.
fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Read an identifier from `s` starting at position 0. Returns the identifier
/// and how many bytes were consumed.
fn read_ident(s: &[u8]) -> Option<(&[u8], usize)> {
    if s.is_empty() || !is_ident_char(s[0]) {
        return None;
    }
    let len = s.iter().take_while(|&&c| is_ident_char(c)).count();
    Some((&s[..len], len))
}

/// Skip ASCII whitespace from the front of `s`. Returns remaining bytes and
/// number skipped.
fn skip_ws(s: &[u8]) -> (&[u8], usize) {
    let n = s.iter().take_while(|&&c| c == b' ' || c == b'\t').count();
    (&s[n..], n)
}

/// Attempt to parse `lhs OP rhs ? branch_a : branch_b` starting at offset
/// `start` in `bytes`. If the comparison operands match the ternary branches
/// in the expected way, return the byte range of the whole expression.
///
/// Returns `Some((start, end))` if a min/max ternary is detected.
fn try_parse_minmax(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let s = &bytes[start..];

    // Read lhs identifier
    let (lhs, lhs_len) = read_ident(s)?;
    let mut pos = lhs_len;

    // Skip whitespace
    let (s2, ws1) = skip_ws(&s[pos..]);
    pos += ws1;

    // Read operator: `>=`, `<=`, `>`, `<`
    let op: &[u8];
    if s2.starts_with(b">=") {
        op = b">=";
        pos += 2;
    } else if s2.starts_with(b"<=") {
        op = b"<=";
        pos += 2;
    } else if s2.starts_with(b">") && !s2.starts_with(b">=") {
        op = b">";
        pos += 1;
    } else if s2.starts_with(b"<") && !s2.starts_with(b"<=") {
        op = b"<";
        pos += 1;
    } else {
        return None;
    }
    let _ = op; // operator captured for semantics but validated through branch matching

    // Skip whitespace
    let (_, ws2) = skip_ws(&s[pos..]);
    pos += ws2;

    // Read rhs identifier
    let (rhs, rhs_len) = read_ident(&s[pos..])?;
    pos += rhs_len;

    // Skip whitespace
    let (_, ws3) = skip_ws(&s[pos..]);
    pos += ws3;

    // Expect `?`
    if s[pos..].first() != Some(&b'?') {
        return None;
    }
    pos += 1;

    // Skip whitespace
    let (_, ws4) = skip_ws(&s[pos..]);
    pos += ws4;

    // Read branch_a identifier
    let (branch_a, ba_len) = read_ident(&s[pos..])?;
    pos += ba_len;

    // Skip whitespace
    let (_, ws5) = skip_ws(&s[pos..]);
    pos += ws5;

    // Expect `:`
    if s[pos..].first() != Some(&b':') {
        return None;
    }
    pos += 1;

    // Skip whitespace
    let (_, ws6) = skip_ws(&s[pos..]);
    pos += ws6;

    // Read branch_b identifier
    let (branch_b, bb_len) = read_ident(&s[pos..])?;
    pos += bb_len;

    // Validate: branches must match the operands (in any order that forms min/max)
    // Valid patterns:
    //   a > b ? a : b  (max)   a >= b ? a : b  (max)
    //   a < b ? a : b  (min)   a <= b ? a : b  (min)
    //   a > b ? b : a  (min)   a >= b ? b : a  (min)
    //   a < b ? b : a  (max)   a <= b ? b : a  (max)
    let branches_match = (branch_a == lhs && branch_b == rhs)
        || (branch_a == rhs && branch_b == lhs);

    if !branches_match {
        return None;
    }

    Some((start, start + pos))
}

/// Returns true if byte position `pos` in `line` is inside a string literal
/// or past the start of a comment.
fn is_in_string_or_comment(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;

    while i < pos && i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => {
                return true;
            }
            None => {}
        }
        i += 1;
    }

    in_str.is_some()
}

impl Rule for MinMaxComparison {
    fn name(&self) -> &'static str {
        "Style/MinMaxComparison"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Quick pre-screen: must contain `?` to be a ternary
            if !line.contains('?') {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Scan each byte position as a potential start of an identifier
            let mut search = 0usize;
            while search < bytes.len() {
                // Only attempt parse at identifier start characters
                if is_ident_char(bytes[search]) && (search == 0 || !is_ident_char(bytes[search - 1])) {
                    if !is_in_string_or_comment(bytes, search) {
                        if let Some((rel_start, rel_end)) = try_parse_minmax(bytes, search) {
                            let start = (line_start + rel_start) as u32;
                            let end = (line_start + rel_end) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use min or max instead of ternary comparison.".into(),
                                range: TextRange::new(start, end),
                                severity: Severity::Warning,
                            });
                            // Skip past this match to avoid re-scanning inside it
                            search = rel_end;
                            continue;
                        }
                    }
                }
                search += 1;
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_max_ternary_with_greater_than() {
        let src = "result = a > b ? a : b\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'a > b ? a : b', got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/MinMaxComparison"));
    }

    #[test]
    fn detects_min_ternary_with_less_than() {
        let src = "result = x < y ? x : y\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'x < y ? x : y', got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/MinMaxComparison"));
    }

    #[test]
    fn detects_max_ternary_with_greater_than_or_equal() {
        let src = "val = foo >= bar ? foo : bar\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'foo >= bar ? foo : bar', got none"
        );
    }

    #[test]
    fn detects_min_ternary_with_less_than_or_equal() {
        let src = "val = foo <= bar ? foo : bar\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violation for 'foo <= bar ? foo : bar', got none"
        );
    }

    #[test]
    fn no_violation_for_non_min_max_ternary() {
        let src = "a > b ? do_something : do_other\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation for non-min/max ternary, got: {:?}",
            diags
        );
    }

    #[test]
    fn no_violation_for_array_max() {
        let src = "result = [a, b].max\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violations, got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_different_operands_in_branches() {
        // a > b ? c : d — branches don't match operands
        let src = "result = a > b ? c : d\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violation when branches don't match operands, got: {:?}",
            diags
        );
    }

    #[test]
    fn uses_offending_fixture() {
        let offending =
            include_str!("../../tests/fixtures/style/min_max_comparison/offending.rb");
        let ctx = LintContext::new(Path::new("test.rb"), offending);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            !diags.is_empty(),
            "expected violations in offending.rb, got none"
        );
        assert!(diags.iter().all(|d| d.rule == "Style/MinMaxComparison"));
    }

    #[test]
    fn no_violation_on_passing_fixture() {
        let passing =
            include_str!("../../tests/fixtures/style/min_max_comparison/passing.rb");
        let ctx = LintContext::new(Path::new("test.rb"), passing);
        let diags = MinMaxComparison.check_source(&ctx);
        assert!(
            diags.is_empty(),
            "expected no violations in passing.rb, got: {:?}",
            diags
        );
    }
}
