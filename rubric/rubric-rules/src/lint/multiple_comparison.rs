use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashSet;

pub struct MultipleComparison;

/// Given a percent-literal opener character, return the matching closer.
/// e.g. `(` → `)`, `[` → `]`, `{` → `}`, `<` → `>`, any other char → same char.
fn percent_literal_closer(opener: char) -> char {
    match opener {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        c   => c,
    }
}

/// Strip string literals (single- and double-quoted), percent literals
/// (%(...), %w[...], %i[...], etc.), and inline comments from a line,
/// replacing their content with spaces so that character positions are preserved.
/// This prevents `<`, `>` inside strings, percent literals, or comments from
/// being treated as comparison operators.
fn sanitise_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut out: Vec<char> = chars.clone();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Inline comment — blank the rest of the line.
        // Guard: `#` inside a string or percent literal is handled by those branches
        // skipping past their content before we loop back here.
        if ch == '#' {
            for j in i..len {
                out[j] = ' ';
            }
            break;
        }

        // Percent literal: `%` followed by an optional type letter then a delimiter.
        // We handle it before single/double quote so that e.g. `%q(...)` is covered.
        if ch == '%' && i + 1 < len {
            // Skip optional type letter (w, W, i, I, q, Q, r, x, s, or none)
            let next = chars[i + 1];
            let (type_end, delim_char) = if next.is_ascii_alphabetic() && i + 2 < len {
                (i + 2, chars[i + 2])
            } else if !next.is_alphanumeric() && next != ' ' && next != '\n' {
                (i + 1, next)
            } else {
                // Not a percent literal we can parse, move on
                i += 1;
                continue;
            };

            let closer = percent_literal_closer(delim_char);
            // Blank everything from % up to and including the opening delimiter
            for j in i..=type_end {
                out[j] = ' ';
            }
            i = type_end + 1;

            // For `%<...>` with `<` as the delimiter, closer is `>`.
            // For balanced delimiters we track nesting depth.
            let balanced = matches!(delim_char, '(' | '[' | '{' | '<');
            let mut depth: u32 = 1;

            while i < len {
                let c = chars[i];
                if balanced && c == delim_char {
                    depth += 1;
                    out[i] = ' ';
                } else if c == closer {
                    depth -= 1;
                    out[i] = ' ';
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                } else {
                    out[i] = ' ';
                }
                i += 1;
            }
            continue;
        }

        // Single-quoted string
        if ch == '\'' {
            out[i] = ' '; // opening quote
            i += 1;
            while i < len && chars[i] != '\'' {
                if chars[i] == '\\' && i + 1 < len {
                    out[i] = ' ';
                    i += 1;
                    out[i] = ' ';
                } else {
                    out[i] = ' ';
                }
                i += 1;
            }
            if i < len {
                out[i] = ' '; // closing quote
            }
            i += 1;
            continue;
        }

        // Double-quoted string
        if ch == '"' {
            out[i] = ' '; // opening quote
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    out[i] = ' ';
                    i += 1;
                    out[i] = ' ';
                } else {
                    out[i] = ' ';
                }
                i += 1;
            }
            if i < len {
                out[i] = ' '; // closing quote
            }
            i += 1;
            continue;
        }

        i += 1;
    }

    out.iter().collect()
}

/// Collect (byte_offset, operator_str) pairs for standalone comparison operators in
/// a sanitised line. Skips `<<`/`>>` (shift/shovel) and `=>` (hash rocket).
fn comparison_positions(line: &str) -> Vec<(usize, &'static str)> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut result = Vec::new();
    let mut i = 0;

    while i < len {
        let b = bytes[i];

        if b == b'<' {
            let next = bytes.get(i + 1).copied();
            if next == Some(b'<') {
                // `<<` — shovel / left-shift, skip both chars
                i += 2;
                continue;
            }
            if next == Some(b'=') {
                // `<=`
                result.push((i, "<="));
                i += 2;
                continue;
            }
            // bare `<`
            result.push((i, "<"));
            i += 1;
            continue;
        }

        if b == b'>' {
            let next = bytes.get(i + 1).copied();
            let prev = if i > 0 { bytes.get(i - 1).copied() } else { None };
            if next == Some(b'>') {
                // `>>` — right-shift, skip both
                i += 2;
                continue;
            }
            if next == Some(b'=') {
                // `>=`
                result.push((i, ">="));
                i += 2;
                continue;
            }
            // bare `>` — skip if preceded by `=` (hash rocket `=>`)
            if prev == Some(b'=') {
                i += 1;
                continue;
            }
            result.push((i, ">"));
            i += 1;
            continue;
        }

        i += 1;
    }

    result
}

impl Rule for MultipleComparison {
    fn name(&self) -> &'static str {
        "Lint/MultipleComparison"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut seen_lines: HashSet<usize> = HashSet::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let sanitised = sanitise_line(line);
            let positions = comparison_positions(&sanitised);

            // A chained comparison requires at least two operators with something
            // (non-empty, no boolean connective) between consecutive pairs.
            if positions.len() < 2 {
                continue;
            }

            let mut found_chain = false;
            'outer: for idx in 0..positions.len() - 1 {
                let (pos1, op1) = positions[idx];
                let (pos2, _)   = positions[idx + 1];

                let op1_end = pos1 + op1.len();
                if pos2 <= op1_end {
                    continue;
                }

                let between = sanitised[op1_end..pos2].trim();
                if between.is_empty() {
                    continue;
                }
                // Boolean connectives between operators indicate separate comparisons,
                // not a chained one.
                if between.contains('&') || between.contains('|') {
                    continue;
                }

                found_chain = true;
                break 'outer;
            }

            if found_chain && seen_lines.insert(i) {
                let indent = line.len() - trimmed.len();
                let line_start = ctx.line_start_offsets[i] as usize;
                let pos = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Chained comparison does not work as expected in Ruby.".into(),
                    range: TextRange::new(pos, pos + trimmed.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
