use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SlicingWithRange;

impl Rule for SlicingWithRange {
    fn name(&self) -> &'static str {
        "Style/SlicingWithRange"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            // Skip comment lines
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            // Strip inline comment
            let code = strip_inline_comment(line);

            // Look for patterns: `..identifier.length - 1]` or `..identifier.size - 1]`
            // We scan for `..` then check what follows.
            if let Some(violation_col) = find_slicing_violation(code) {
                let line_start = ctx.line_start_offsets[i];
                let start = line_start + violation_col as u32;
                let end = line_start + code.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Prefer `[n..]` over `[n..length - 1]`.".into(),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}

/// Scan a line for the pattern `..IDENTIFIER.length - 1]` or `..IDENTIFIER.size - 1]`.
/// Returns the byte column of the `..` if found, or None.
fn find_slicing_violation(code: &str) -> Option<usize> {
    let bytes = code.as_bytes();
    let n = bytes.len();

    let mut i = 0;
    while i + 1 < n {
        // Look for `..`
        if bytes[i] == b'.' && bytes[i + 1] == b'.' {
            let dot_dot_pos = i;
            let after_dots = i + 2;

            // Check if the next part matches `IDENTIFIER.length - 1]` or `IDENTIFIER.size - 1]`
            if is_length_or_size_minus_one(&bytes[after_dots..]) {
                return Some(dot_dot_pos);
            }

            i += 2;
        } else {
            i += 1;
        }
    }

    None
}

/// Check if a byte slice starts with `IDENTIFIER.length - 1]` or `IDENTIFIER.size - 1]`.
fn is_length_or_size_minus_one(rest: &[u8]) -> bool {
    // Skip optional identifier (variable name): alphanumeric + underscore
    let mut pos = 0;
    // The identifier may be empty if we're looking at something like `.length - 1]`
    // but typically it has at least one char
    while pos < rest.len() && (rest[pos].is_ascii_alphanumeric() || rest[pos] == b'_') {
        pos += 1;
    }

    // Must have at least one identifier character
    if pos == 0 {
        return false;
    }

    // Must be followed by `.`
    if pos >= rest.len() || rest[pos] != b'.' {
        return false;
    }
    pos += 1;

    // Must be followed by `length` or `size`
    let keyword_match = if rest[pos..].starts_with(b"length") {
        Some(6)
    } else if rest[pos..].starts_with(b"size") {
        Some(4)
    } else {
        None
    };

    let keyword_len = match keyword_match {
        Some(l) => l,
        None => return false,
    };
    pos += keyword_len;

    // After `length`/`size`, must not be a word character (e.g. `lengths` should not match)
    if pos < rest.len() && (rest[pos].is_ascii_alphanumeric() || rest[pos] == b'_') {
        return false;
    }

    // Skip optional whitespace
    while pos < rest.len() && rest[pos] == b' ' {
        pos += 1;
    }

    // Must be followed by `- 1]` (with optional spaces around `-`)
    if pos >= rest.len() || rest[pos] != b'-' {
        return false;
    }
    pos += 1;

    // Skip whitespace
    while pos < rest.len() && rest[pos] == b' ' {
        pos += 1;
    }

    // Must be `1`
    if pos >= rest.len() || rest[pos] != b'1' {
        return false;
    }
    pos += 1;

    // After `1`, skip whitespace and check for `]`
    while pos < rest.len() && rest[pos] == b' ' {
        pos += 1;
    }

    if pos < rest.len() && rest[pos] == b']' {
        return true;
    }

    false
}

/// Strip inline comment from a line, respecting string literals.
fn strip_inline_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut in_string: Option<u8> = None;
    let mut j = 0;
    while j < n {
        let b = bytes[j];
        if let Some(delim) = in_string {
            if b == b'\\' {
                j += 2;
                continue;
            }
            if b == delim {
                in_string = None;
            }
        } else {
            match b {
                b'"' | b'\'' | b'`' => in_string = Some(b),
                b'#' => return &line[..j],
                _ => {}
            }
        }
        j += 1;
    }
    line
}
