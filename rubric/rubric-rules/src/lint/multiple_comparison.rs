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
/// (%(...), %w[...], %i[...], etc.), regex literals (/…/), and inline
/// comments from a line, replacing their content with spaces so that
/// character positions are preserved.
/// This prevents `<`, `>` inside strings, percent literals, regex, or comments
/// from being treated as comparison operators.
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

        // Regex literal: `/pattern/flags`
        // We detect `/` when preceded only by whitespace, an operator, or
        // the start of the line — i.e. it is a regex opener, not division.
        // Simple heuristic: if the previous non-space char is one of
        // `=`, `(`, `,`, `[`, `!`, `&`, `|`, `{`, `;`, or start-of-line,
        // treat this `/` as beginning a regex literal and blank until the
        // closing unescaped `/`.
        if ch == '/' {
            let prev_non_space = (0..i)
                .rev()
                .find(|&j| chars[j] != ' ' && chars[j] != '\t')
                .map(|j| chars[j]);
            let is_regex_start = match prev_non_space {
                None => true, // start of line
                Some(c) => matches!(c, '=' | '(' | ',' | '[' | '!' | '&' | '|' | '{' | ';' | ':' | '<' | '>' | '+' | '-' | '*' | '%' | '^' | '~' | '?' | '\n'),
            };
            if is_regex_start {
                out[i] = ' '; // opening slash
                i += 1;
                while i < len && chars[i] != '/' {
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
                    out[i] = ' '; // closing slash
                }
                i += 1;
                // Skip any trailing regex flags (i, m, x, etc.)
                while i < len && chars[i].is_ascii_alphabetic() {
                    out[i] = ' ';
                    i += 1;
                }
                continue;
            }
        }

        i += 1;
    }

    out.iter().collect()
}

/// Extract the heredoc terminator word from a line that opens a heredoc.
///
/// Handles `<<~WORD`, `<<-WORD`, and `<<WORD` (bare). Returns `None` if
/// no heredoc opener is found on the line.
///
/// Quoted heredocs (e.g. `<<"WORD"` or `<<'WORD'`) are also handled by
/// stripping the surrounding quote characters.
fn heredoc_terminator(line: &str) -> Option<String> {
    let pos = line.find("<<")?;
    let rest = &line[pos + 2..];

    // Strip optional `-` or `~` sigil.
    let rest = rest.strip_prefix('-').unwrap_or(rest);
    let rest = rest.strip_prefix('~').unwrap_or(rest);

    // Strip optional surrounding quotes.
    let rest = if (rest.starts_with('"') && rest.contains('"'))
        || (rest.starts_with('\'') && rest.contains('\''))
        || (rest.starts_with('`') && rest.contains('`'))
    {
        &rest[1..]
    } else {
        rest
    };

    // Collect the identifier (letters, digits, underscores).
    let word: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if word.is_empty() {
        None
    } else {
        Some(word)
    }
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
        // When `Some(word)`, we are inside a heredoc whose terminator is `word`.
        let mut heredoc_term: Option<String> = None;
        // When true, we are inside a multiline regex (CONST = / ... /flags).
        // Lines inside a multiline regex look like Ruby code but are regex patterns;
        // `<` and `>` are named-capture-group delimiters, not comparison operators.
        let mut in_multiline_regex = false;

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let t = trimmed.trim();

            // If we are inside a heredoc body, skip until we find the terminator.
            if let Some(ref term) = heredoc_term {
                if t == term.as_str() {
                    heredoc_term = None;
                }
                // Either way (terminator line or body line), skip comparison check.
                continue;
            }

            // Skip multiline regex body lines. Exit when we hit the closing `/flags`.
            if in_multiline_regex {
                // Closing line: just `/` optionally followed by regex flags and whitespace
                let t2 = t.trim_start_matches(|c: char| c == ' ' || c == '\t');
                if t2.starts_with('/') {
                    let after_slash = t2[1..].trim_start_matches(|c: char| c.is_ascii_alphabetic());
                    if after_slash.is_empty() || after_slash.starts_with('#') || after_slash.trim().is_empty() {
                        in_multiline_regex = false;
                    }
                }
                continue;
            }

            if trimmed.starts_with('#') {
                continue;
            }

            // Check whether this line opens a heredoc; record terminator for
            // subsequent lines. The opening line itself is still checked for
            // chained comparisons (the `<<~WORD` expression is valid Ruby).
            if let Some(term) = heredoc_terminator(line) {
                heredoc_term = Some(term);
            }

            // Detect multiline regex opener: line ends with `= /` (no closing `/`).
            // The regex CONST = /\n  body\n/flags spans multiple lines.
            {
                let raw_trimmed = t;
                let bytes = raw_trimmed.as_bytes();
                // Look for a `/` that is a regex opener and is the last non-whitespace char
                if bytes.last() == Some(&b'/') {
                    // Check if the preceding non-space char is a regex-opener
                    let prev = bytes[..bytes.len()-1].iter().rposition(|&b| b != b' ' && b != b'\t');
                    let is_regex_opener = match prev {
                        Some(p) => matches!(bytes[p], b'=' | b'(' | b',' | b'[' | b'!' | b'&' | b'|' | b'{' | b';' | b':'),
                        None => true,
                    };
                    if is_regex_opener {
                        in_multiline_regex = true;
                    }
                }
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
