use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SortComparison;

/// Returns the byte index of a real `#` comment character in `line`, skipping
/// `#` inside string literals.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut interp_depth: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                interp_depth += 1;
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d && interp_depth == 0 => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        if interp_depth > 0 && bytes[i] == b'}' {
            interp_depth -= 1;
        }
        i += 1;
    }
    None
}

/// Returns `true` if `pos` in `bytes` is inside a string literal.
fn in_string_at(bytes: &[u8], pos: usize) -> bool {
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
            None if bytes[i] == b'#' => return false,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Checks whether the block content (the text between `{` and `}` after `.sort`)
/// contains a `<=>` spaceship operator, indicating a trivial ascending comparison.
///
/// The block must:
/// 1. Have exactly two block parameters separated by `,`
/// 2. Contain `<=>` between those two parameters (in the same order, ascending)
///
/// Returns `true` only for the pattern `|a, b| a <=> b` (ascending, same order).
fn is_trivial_sort_block(block_content: &str) -> bool {
    // block_content is the text between `{` and `}` (exclusive)
    let content = block_content.trim();

    // Must start with `|`
    let rest = match content.strip_prefix('|') {
        Some(r) => r,
        None => return false,
    };

    // Find closing `|`
    let pipe_end = match rest.find('|') {
        Some(p) => p,
        None => return false,
    };

    let params_str = &rest[..pipe_end];
    let body = rest[pipe_end + 1..].trim();

    // Parse two params separated by `,`
    let params: Vec<&str> = params_str.split(',').map(str::trim).collect();
    if params.len() != 2 {
        return false;
    }
    let lhs = params[0];
    let rhs = params[1];

    // Body must be `lhs <=> rhs` — the ascending, identity comparison
    let expected = format!("{} <=> {}", lhs, rhs);
    body == expected.as_str()
}

impl Rule for SortComparison {
    fn name(&self) -> &'static str {
        "Style/SortComparison"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        const MESSAGE: &str =
            "Use Array#sort without a block instead of sort { |a, b| a <=> b }.";

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];
            let bytes = scan_slice.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Look for `.sort` followed by optional whitespace then `{`
            let sort_pattern = b".sort";
            let mut search = 0usize;

            while search < bytes.len() {
                let found = bytes[search..]
                    .windows(sort_pattern.len())
                    .position(|w| w == sort_pattern);

                let rel = match found {
                    Some(r) => r,
                    None => break,
                };
                let abs = search + rel;

                // Verify it's `.sort` and not `.sort_by` or `.sort_something`
                let after_sort = abs + sort_pattern.len();
                let next_byte = bytes.get(after_sort).copied();
                let is_sort_method = match next_byte {
                    // `.sort{` or `.sort ` or `.sort(` or end of scan
                    None | Some(b'{') | Some(b' ') | Some(b'\t') | Some(b'(') => true,
                    // `.sort_by`, `.sort_abc` — skip
                    Some(b'_') | Some(b'a'..=b'z') | Some(b'A'..=b'Z') => false,
                    _ => true,
                };

                if is_sort_method && !in_string_at(bytes, abs) {
                    // Find the opening `{` after `.sort`
                    let brace_search_start = after_sort;
                    let brace_pos = bytes[brace_search_start..]
                        .iter()
                        .position(|&b| b == b'{')
                        .map(|p| brace_search_start + p);

                    if let Some(brace_abs) = brace_pos {
                        // Verify only whitespace between `.sort` and `{`
                        let between = &scan_slice[after_sort..brace_abs];
                        if between.chars().all(|c| c.is_whitespace()) {
                            // Find matching closing `}`
                            let block_start = brace_abs + 1;
                            let close_brace = bytes[block_start..]
                                .iter()
                                .position(|&b| b == b'}')
                                .map(|p| block_start + p);

                            if let Some(close_abs) = close_brace {
                                let block_content =
                                    &scan_slice[block_start..close_abs];

                                if is_trivial_sort_block(block_content) {
                                    // Flag the `.sort` portion
                                    let start = (line_start + abs) as u32;
                                    let end = (line_start + close_abs + 1) as u32;
                                    diags.push(Diagnostic {
                                        rule: self.name(),
                                        message: MESSAGE.into(),
                                        range: TextRange::new(start, end),
                                        severity: Severity::Warning,
                                    });
                                }
                            }
                        }
                    }
                }

                search = abs + sort_pattern.len();
            }
        }

        diags
    }
}
