use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MapCompactWithConditionalBlock;

impl Rule for MapCompactWithConditionalBlock {
    fn name(&self) -> &'static str {
        "Style/MapCompactWithConditionalBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect single-line: .map { ... }.compact or .map { ... }.compact!
            // Pattern: \.map\s*\{.*\}\s*\.compact
            if let Some(diag) = detect_single_line(ctx, i, line) {
                diags.push(diag);
            }
        }

        diags
    }
}

fn detect_single_line(ctx: &LintContext, line_idx: usize, line: &str) -> Option<Diagnostic> {
    let bytes = line.as_bytes();
    let mut search = 0;

    while search < bytes.len() {
        // Look for `.map` token
        let map_rel = match bytes[search..].windows(4).position(|w| w == b".map") {
            Some(r) => r,
            None => break,
        };
        let map_pos = search + map_rel;

        // Ensure `.map` is followed by a word boundary (not `.map_something`)
        let after_map = map_pos + 4;
        if after_map >= bytes.len() {
            break;
        }
        let next = bytes[after_map];
        if next.is_ascii_alphanumeric() || next == b'_' {
            search = map_pos + 4;
            continue;
        }

        // Find the `{` that opens the block after `.map`
        let brace_pos = match find_brace_after(bytes, after_map) {
            Some(p) => p,
            None => {
                search = map_pos + 4;
                continue;
            }
        };

        // Find the matching `}` for that `{`
        let close_pos = match find_matching_close_brace(bytes, brace_pos) {
            Some(p) => p,
            None => {
                search = brace_pos + 1;
                continue;
            }
        };

        // Only flag if the block body contains a conditional expression
        let block_body = &bytes[brace_pos + 1..close_pos];
        if !block_has_conditional(block_body) {
            search = close_pos + 1;
            continue;
        }

        // After `}` look for `.compact` (optionally preceded by whitespace)
        let rest = &bytes[close_pos + 1..];
        let compact_offset = match find_compact(rest) {
            Some(o) => o,
            None => {
                search = close_pos + 1;
                continue;
            }
        };
        let compact_abs = close_pos + 1 + compact_offset;

        // Report at the `.compact` position
        let line_start = ctx.line_start_offsets[line_idx] as usize;
        let start = (line_start + compact_abs) as u32;
        let end = start + 8; // len(".compact")

        return Some(Diagnostic {
            rule: "Style/MapCompactWithConditionalBlock",
            message: "Use filter_map instead of map { ... }.compact.".into(),
            range: TextRange::new(start, end),
            severity: Severity::Warning,
        });
    }

    None
}

/// Returns true if the block body bytes contain an `if`, `unless`, or ternary `?` conditional.
fn block_has_conditional(body: &[u8]) -> bool {
    // Look for ` if ` or ` unless ` or ` ? ` as simple heuristics
    // (preceded and followed by space to reduce false positives from strings)
    for window in body.windows(4) {
        if window == b" if " {
            return true;
        }
    }
    for window in body.windows(8) {
        if window == b" unless " {
            return true;
        }
    }
    // Ternary: look for ` ? ` pattern
    for window in body.windows(3) {
        if window == b" ? " {
            return true;
        }
    }
    false
}

/// Find the index of `{` at or after `from`, skipping whitespace.
fn find_brace_after(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' => i += 1,
            b'{' => return Some(i),
            _ => return None,
        }
    }
    None
}

/// Find the matching `}` for the `{` at position `open`.
/// Handles basic nesting. Does not handle strings inside blocks.
fn find_matching_close_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0u32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find `.compact` (optionally `.compact!`) at the start of `rest`, skipping whitespace.
/// Returns the byte offset within `rest` where `.compact` begins.
fn find_compact(rest: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            b' ' | b'\t' => i += 1,
            b'.' => {
                // Check for `compact` after the dot
                let after_dot = i + 1;
                if rest[after_dot..].starts_with(b"compact") {
                    // Ensure word boundary: next char after `compact` must not be alpha/_
                    let after_compact = after_dot + 7;
                    let boundary_ok = after_compact >= rest.len()
                        || {
                            let c = rest[after_compact];
                            !c.is_ascii_alphanumeric() && c != b'_'
                        };
                    if boundary_ok {
                        return Some(i);
                    }
                }
                return None;
            }
            _ => return None,
        }
    }
    None
}
