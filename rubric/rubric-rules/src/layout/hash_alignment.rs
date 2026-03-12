use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct HashAlignment;

/// Returns the byte position of `=>` on this line, searching outside
/// string literals and inline comments. Returns `None` if not found.
fn find_rocket(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut j = 0;
    let mut in_string: Option<u8> = None;

    while j < len {
        let b = bytes[j];
        match in_string {
            Some(_) if b == b'\\' => { j += 2; continue; }
            Some(delim) if b == delim => { in_string = None; j += 1; continue; }
            Some(_) => { j += 1; continue; }
            None if b == b'"' || b == b'\'' => { in_string = Some(b); j += 1; continue; }
            None if b == b'#' => break,
            None => {}
        }
        if j + 1 < len && bytes[j] == b'=' && bytes[j + 1] == b'>' {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Counts spaces immediately before `pos` in `line`.
fn spaces_before(line: &str, pos: usize) -> usize {
    let bytes = line.as_bytes();
    let mut k = pos as isize - 1;
    let mut count = 0usize;
    while k >= 0 && bytes[k as usize] == b' ' {
        count += 1;
        k -= 1;
    }
    count
}

/// Returns `true` if the rocket at `rocket_pos` on line `idx` is part of a
/// table-aligned group: at least one neighboring line (searching up to 15
/// lines in each direction, skipping blank/comment lines) has a rocket at
/// the same column.
///
/// This allows `table`-style hashes where all `=>` are aligned to the same
/// column, while still flagging lone misaligned rockets.
/// Returns true if a trimmed line is just a hash/array/paren closer (no key/value).
/// Examples: `},`, `}`, `],`, `]`, `),`, `)`, `}.freeze`, `},\n` etc.
fn is_closer_line(trimmed: &str) -> bool {
    let t = trimmed.trim_end_matches(|c: char| c == ',' || c == ';' || c.is_ascii_whitespace());
    let t = t.trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '.' || c == '(')
             .trim_end_matches(|c: char| c == ',' || c == ';' || c.is_ascii_whitespace());
    // After removing trailing decorations, if what's left is only closing brackets → closer
    t.chars().all(|c| matches!(c, '}' | ']' | ')' | '>' | '#' | ' ' | '\t'))
        && !t.is_empty()
}

fn is_table_aligned(rockets: &[Option<usize>], lines: &[&str], idx: usize, rocket_pos: usize) -> bool {
    let n = rockets.len();
    let my_indent = lines[idx].len() - lines[idx].trim_start().len();

    // Search backward
    let start = if idx > 15 { idx - 15 } else { 0 };
    for k in (start..idx).rev() {
        let line = lines[k];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let k_indent = line.len() - line.trim_start().len();
        match rockets[k] {
            Some(p) if p == rocket_pos => return true, // same column — part of group
            Some(_) if k_indent > my_indent => continue, // inner scope, different col — skip
            Some(_) => break, // same or outer scope, different column — not same group
            None if k_indent > my_indent => continue, // continuation/value line — skip
            None if is_closer_line(trimmed) => continue, // closer line — skip
            None => break, // group boundary at same/outer scope
        }
    }

    // Search forward
    let end = (idx + 16).min(n);
    for k in (idx + 1)..end {
        let line = lines[k];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let k_indent = line.len() - line.trim_start().len();
        match rockets[k] {
            Some(p) if p == rocket_pos => return true,
            Some(_) if k_indent > my_indent => continue,
            Some(_) => break,
            None if k_indent > my_indent => continue,
            None if is_closer_line(trimmed) => continue,
            None => break,
        }
    }

    false
}

impl Rule for HashAlignment {
    fn name(&self) -> &'static str {
        "Layout/HashAlignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let lines = &ctx.lines;
        let n = lines.len();

        // Pass 1: find `=>` position for every line (None = no rocket on line).
        let rockets: Vec<Option<usize>> = lines.iter().map(|l| {
            if l.trim_start().starts_with('#') { return None; }
            find_rocket(l)
        }).collect();

        let mut diags = Vec::new();

        for i in 0..n {
            let Some(rocket_pos) = rockets[i] else { continue };

            let spaces = spaces_before(lines[i], rocket_pos);
            if spaces <= 1 {
                continue; // correctly spaced or no leading space
            }

            // Table-style: if this rocket is part of an aligned group where
            // all neighbors have rockets at the same column, allow it.
            if is_table_aligned(&rockets, lines, i, rocket_pos) {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let pos = (line_start + rocket_pos) as u32;
            diags.push(Diagnostic {
                rule: self.name(),
                message: format!(
                    "Hash rocket `=>` has {} spaces before it; expected 1.",
                    spaces
                ),
                range: TextRange::new(pos, pos + 2),
                severity: Severity::Warning,
            });
        }

        diags
    }
}
