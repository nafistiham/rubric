use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RedundantSortBy;

/// Returns the byte index of a real `#` comment character in `line`, skipping
/// `#` inside string literals or interpolations.
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

/// Checks whether the block content between `{` and `}` is a trivial identity block:
/// `|x| x` — same single identifier for param and body.
fn is_identity_sort_by_block(block_content: &str) -> bool {
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

    let param_str = rest[..pipe_end].trim();
    let body = rest[pipe_end + 1..].trim();

    // Single parameter only (no comma)
    if param_str.contains(',') {
        return false;
    }

    // Param must be a valid identifier
    if param_str.is_empty()
        || !param_str
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
    {
        return false;
    }

    // Body must be exactly the param name
    body == param_str
}

impl Rule for RedundantSortBy {
    fn name(&self) -> &'static str {
        "Style/RedundantSortBy"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        const MESSAGE: &str = "Use sort instead of sort_by { |x| x }.";
        const SORT_BY: &[u8] = b".sort_by";

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];
            let bytes = scan_slice.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut search = 0usize;
            while search < bytes.len() {
                let found = bytes[search..]
                    .windows(SORT_BY.len())
                    .position(|w| w == SORT_BY);

                let rel = match found {
                    Some(r) => r,
                    None => break,
                };
                let abs = search + rel;

                // Verify `.sort_by` is followed by optional whitespace then `{`
                let after_sort_by = abs + SORT_BY.len();

                // Ensure `.sort_by` is not inside a string
                if in_string_at(bytes, abs) {
                    search = abs + SORT_BY.len();
                    continue;
                }

                // Find opening `{`
                let brace_pos = bytes[after_sort_by..]
                    .iter()
                    .position(|&b| b == b'{')
                    .map(|p| after_sort_by + p);

                if let Some(brace_abs) = brace_pos {
                    // Only whitespace between `.sort_by` and `{`
                    let between = &scan_slice[after_sort_by..brace_abs];
                    if between.chars().all(|c| c.is_whitespace()) {
                        // Find matching closing `}`
                        let block_start = brace_abs + 1;
                        let close_brace = bytes[block_start..]
                            .iter()
                            .position(|&b| b == b'}')
                            .map(|p| block_start + p);

                        if let Some(close_abs) = close_brace {
                            let block_content = &scan_slice[block_start..close_abs];

                            if is_identity_sort_by_block(block_content) {
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

                search = abs + SORT_BY.len();
            }
        }

        diags
    }
}
