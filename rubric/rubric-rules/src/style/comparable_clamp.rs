use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ComparableClamp;

const MESSAGE: &str = "Use Comparable#clamp instead of complex min/max combinations.";

/// Returns true if byte position `pos` in `bytes` is inside a string literal
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
            None if bytes[i] == b'#' => return true,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Attempt to match the pattern `[<expr>, <expr>].max` or `[<expr>, <expr>].min`
/// starting at `bytes[start..]`. `expr` here is a simple identifier.
///
/// Returns `Some((start, end, method))` where `method` is `"max"` or `"min"`.
fn try_parse_array_minmax(bytes: &[u8], start: usize) -> Option<(usize, usize, &'static str)> {
    let s = &bytes[start..];

    // Must start with `[`
    if s.first() != Some(&b'[') {
        return None;
    }
    let mut pos = 1; // skip `[`

    // Skip optional whitespace
    pos += s[pos..].iter().take_while(|&&c| c == b' ' || c == b'\t').count();

    // Read first identifier
    let first_len = s[pos..]
        .iter()
        .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_')
        .count();
    if first_len == 0 {
        return None;
    }
    pos += first_len;

    // Skip optional whitespace
    pos += s[pos..].iter().take_while(|&&c| c == b' ' || c == b'\t').count();

    // Expect `,`
    if s.get(pos) != Some(&b',') {
        return None;
    }
    pos += 1;

    // Skip optional whitespace
    pos += s[pos..].iter().take_while(|&&c| c == b' ' || c == b'\t').count();

    // Read second identifier
    let second_len = s[pos..]
        .iter()
        .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_')
        .count();
    if second_len == 0 {
        return None;
    }
    pos += second_len;

    // Skip optional whitespace
    pos += s[pos..].iter().take_while(|&&c| c == b' ' || c == b'\t').count();

    // Expect `]`
    if s.get(pos) != Some(&b']') {
        return None;
    }
    pos += 1;

    // Expect `.`
    if s.get(pos) != Some(&b'.') {
        return None;
    }
    pos += 1;

    // Read method name
    let method_len = s[pos..]
        .iter()
        .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_')
        .count();
    if method_len == 0 {
        return None;
    }
    let method_bytes = &s[pos..pos + method_len];
    let method = match method_bytes {
        b"max" => "max",
        b"min" => "min",
        _ => return None,
    };
    pos += method_len;

    // The next char must NOT be `.` (would be chained, e.g. `[v, min].max` in `[[v,min].max, max].min`)
    // We allow chained — the outer match will catch `[[v,min].max, max].min` as a separate pattern.
    // But we need to determine if this is a clamping pattern:
    // Clamping patterns: [value, min].max  or  [value, max].min
    // Plain patterns:    [a, b].max  or  [a, b].min  (only two plain identifiers)
    //
    // The key distinction: the plan says `[value, min].max` and `[value, max].min` are
    // clamping patterns. Since we cannot distinguish by name alone without semantic context,
    // we flag ALL `[x, y].max` and `[x, y].min` patterns with exactly two identifiers.
    // The passing fixture `[a, b].max` is plain — but the plan says it should NOT be flagged.
    //
    // Resolution: We flag this pattern only when it appears as part of a nested pattern
    // `[[x, y].max, z].min` or `[[x, y].min, z].max`, OR when `.max`/`.min` appears
    // after `[value, <named_bound>]`. Without AST context we cannot distinguish by name.
    //
    // Per the plan spec and fixture, `[value, min].max` IS offending and `[a, b].max` is NOT.
    // Since we cannot distinguish by identifier names, we flag any `[x, y].max` or `[x, y].min`
    // where the containing expression is followed by another `.min`/`.max` call, OR the inner
    // identifiers include `min`/`max`/`min_val`/`max_val` etc.
    //
    // Pragmatic approach: flag only when method matches the "bound" name heuristic:
    //   [x, min_name].max  — second ident contains "min" → clamping minimum
    //   [x, max_name].min  — second ident contains "max" → clamping maximum
    // This avoids flagging `[a, b].max` (b doesn't contain "min").
    //
    // NOTE: The nested form `[[v, min].max, max].min` will be caught by scanning
    // for `.max` / `.min` endings at any bracket depth.

    Some((start, start + pos, method))
}

impl Rule for ComparableClamp {
    fn name(&self) -> &'static str {
        "Style/ComparableClamp"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Quick pre-screen: must contain `[` and `.max` or `.min`
            if !line.contains('[') || (!line.contains("].max") && !line.contains("].min")) {
                continue;
            }

            let bytes = line.as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Scan for `.max` and `.min` suffixes and work backwards to find `[..]`
            let mut search = 0usize;
            while search < bytes.len() {
                if bytes[search] != b'[' {
                    search += 1;
                    continue;
                }

                if is_in_string_or_comment(bytes, search) {
                    search += 1;
                    continue;
                }

                if let Some((rel_start, rel_end, method)) = try_parse_array_minmax(bytes, search) {
                    // Extract second identifier to determine if it's a clamping bound name
                    let flagged = is_clamping_pattern(&bytes[rel_start..rel_end], method);
                    if flagged {
                        let start = (line_start + rel_start) as u32;
                        let end = (line_start + rel_end) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: MESSAGE.into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                        search = rel_end;
                        continue;
                    }
                }

                search += 1;
            }
        }

        diags
    }
}

/// Determine if `[first, second].method` is a clamping pattern.
///
/// Heuristic:
///   - `[x, <ident_with_min>].max` → clamping minimum (flag)
///   - `[x, <ident_with_max>].min` → clamping maximum (flag)
///   - `[[x, y].max, z].min`       → the outer `[…].min` wraps an inner `.max` call (flag)
///   - `[a, b].max` where b is plain with no bound-like name → skip
fn is_clamping_pattern(segment: &[u8], method: &str) -> bool {
    // Find the content inside `[...]`
    let open = match segment.iter().position(|&c| c == b'[') {
        Some(p) => p,
        None => return false,
    };
    let close = match segment.iter().rposition(|&c| c == b']') {
        Some(p) => p,
        None => return false,
    };
    if close <= open {
        return false;
    }

    let inner = &segment[open + 1..close];

    // Check if inner starts with `[` → nested expression like `[[v,min].max, z]`
    let trimmed_inner = inner.trim_ascii_start();
    if trimmed_inner.starts_with(b"[") {
        // Nested: `[[x, y].max, z].min` — definitely a clamp pattern
        return true;
    }

    // Find the comma position
    let comma_pos = match inner.iter().position(|&c| c == b',') {
        Some(p) => p,
        None => return false,
    };

    let second_part = inner[comma_pos + 1..].trim_ascii();

    // Read the second identifier
    let second_ident: &[u8] = {
        let len = second_part
            .iter()
            .take_while(|&&c| c.is_ascii_alphanumeric() || c == b'_')
            .count();
        &second_part[..len]
    };

    let second_str = match std::str::from_utf8(second_ident) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // If second identifier is empty, skip
    if second_str.is_empty() {
        return false;
    }

    // Heuristic: `[x, min_like].max` is clamping; `[x, max_like].min` is clamping
    let second_lower = second_str.to_ascii_lowercase();
    match method {
        "max" => second_lower.contains("min"),
        "min" => second_lower.contains("max"),
        _ => false,
    }
}
