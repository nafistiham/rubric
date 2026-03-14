use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct SpaceInsideHashLiteralBraces;

fn heredoc_terminator_sihlb(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            if j < len && (bytes[j] == b'-' || bytes[j] == b'~') { j += 1; }
            if j < len && matches!(bytes[j], b'\'' | b'"' | b'`') { j += 1; }
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') { j += 1; }
            if j > start {
                return Some(line[start..j].to_string());
            }
        }
        i += 1;
    }
    None
}

impl Rule for SpaceInsideHashLiteralBraces {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideHashLiteralBraces"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        let mut in_multiline_regex = false;
        let mut in_percent_regex = false;
        let mut percent_regex_depth = 0usize;
        // Tracks same-char delimiter multiline percent literals (e.g. %r!...!, %r|...|)
        let mut in_same_char_percent: Option<u8> = None;
        let mut in_heredoc: Option<String> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            let bytes = line.as_bytes();
            let len = bytes.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Skip heredoc body lines
            if let Some(ref term) = in_heredoc.clone() {
                if line.trim() == term.as_str() {
                    in_heredoc = None;
                }
                continue;
            }
            // Detect heredoc opener (body starts on next line)
            if let Some(term) = heredoc_terminator_sihlb(line) {
                in_heredoc = Some(term);
                // Fall through: the opener line itself is real Ruby
            }

            // --- Multiline /regex/ body ---
            if in_multiline_regex {
                let mut j = 0;
                while j < len {
                    match bytes[j] {
                        b'\\' => { j += 2; continue; }
                        b'/' => { in_multiline_regex = false; break; }
                        _ => { j += 1; }
                    }
                }
                continue;
            }

            // --- Multiline %r{...} body ---
            if in_percent_regex {
                let mut j = 0;
                while j < len {
                    match bytes[j] {
                        b'\\' => { j += 2; continue; }
                        b'{' => { percent_regex_depth += 1; j += 1; }
                        b'}' => {
                            if percent_regex_depth > 0 {
                                percent_regex_depth -= 1;
                                j += 1;
                            } else {
                                in_percent_regex = false;
                                break;
                            }
                        }
                        _ => { j += 1; }
                    }
                }
                continue;
            }

            // --- Multiline same-char percent literal body (e.g. %r!...!) ---
            if let Some(close) = in_same_char_percent {
                let mut j = 0;
                while j < len {
                    if bytes[j] == b'\\' { j += 2; continue; }
                    if bytes[j] == close { in_same_char_percent = None; break; }
                    j += 1;
                }
                continue;
            }

            let mut in_string: Option<u8> = None;

            let mut j = 0;
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

                // Skip percent literals: %r{...}, %q(...), %Q[...], %w{...}, %(...)
                // etc.  The type letter is optional for %( / %{ / %[ / %<.
                // We detect: `%` followed by optional type letter, then a bracket.
                if b == b'%' && j + 1 < len {
                    let mut k = j + 1;
                    // Optional type letter
                    if k < len && matches!(bytes[k], b'r' | b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b's' | b'x') {
                        k += 1;
                    }
                    if k < len && matches!(bytes[k], b'{' | b'(' | b'[' | b'<' | b'|' | b'!' | b'/' | b'^') {
                        let open_delim = bytes[k];
                        let close_delim = match open_delim {
                            b'{' => b'}',
                            b'(' => b')',
                            b'[' => b']',
                            b'<' => b'>',
                            other => other, // same-char delimiters
                        };
                        let brace_style = close_delim != open_delim;
                        j = k + 1; // advance past the opening delimiter
                        let mut depth = 1usize;
                        while j < len && depth > 0 {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                c if brace_style && c == open_delim => { depth += 1; j += 1; }
                                c if c == close_delim => { depth -= 1; j += 1; }
                                _ => { j += 1; }
                            }
                        }
                        if depth > 0 {
                            if brace_style && open_delim == b'{' {
                                // Unclosed brace-style %r{...} — multiline
                                in_percent_regex = true;
                                percent_regex_depth = depth - 1;
                            } else if !brace_style {
                                // Unclosed same-char delimiter — multiline
                                in_same_char_percent = Some(close_delim);
                            }
                        }
                        continue;
                    }
                }

                // Skip /regex/ literals
                if b == b'/' {
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    if prev == b'=' || prev == b'(' || prev == b','
                        || prev == b'[' || prev == b' ' || prev == b'\t' || prev == 0
                    {
                        j += 1;
                        let mut closed = false;
                        while j < len {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'/' => { closed = true; j += 1; break; }
                                // Skip #{...} interpolation inside regex so inner `/`
                                // doesn't prematurely close the outer regex.
                                b'#' if j + 1 < len && bytes[j + 1] == b'{' => {
                                    j += 2;
                                    let mut depth = 1usize;
                                    while j < len && depth > 0 {
                                        match bytes[j] {
                                            b'\\' => { j += 2; }
                                            b'{' => { depth += 1; j += 1; }
                                            b'}' => { depth -= 1; j += 1; }
                                            _ => { j += 1; }
                                        }
                                    }
                                }
                                _ => { j += 1; }
                            }
                        }
                        if !closed {
                            in_multiline_regex = true;
                        }
                        continue;
                    }
                }

                // Detect `{` not followed by space and not empty `{}`
                if b == b'{' {
                    let next = if j + 1 < len { bytes[j+1] } else { 0 };
                    // Skip empty braces `{}`
                    if next == b'}' {
                        j += 2;
                        continue;
                    }
                    // Skip block braces {|params| body} — these are blocks, not hash literals
                    if next == b'|' {
                        j += 1; // past `{`
                        let mut depth = 1usize;
                        while j < len && depth > 0 {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'{' => { depth += 1; j += 1; }
                                b'}' => { depth -= 1; j += 1; }
                                _ => { j += 1; }
                            }
                        }
                        continue;
                    }
                    // Skip block braces when preceded by `)`, `]`, or an identifier char
                    // (with or without spaces). e.g. `assert_raises(Error){block}`,
                    // `method_name{block}`, `obj.call {block}` — blocks, not hashes.
                    let prev = if j > 0 { bytes[j - 1] } else { 0 };
                    // Find prev non-whitespace char for the space-then-block case
                    let prev_nonws = {
                        let mut k = j;
                        while k > 0 && (bytes[k - 1] == b' ' || bytes[k - 1] == b'\t') { k -= 1; }
                        if k > 0 { bytes[k - 1] } else { 0 }
                    };
                    if matches!(prev, b')' | b']') || prev.is_ascii_alphanumeric() || prev == b'_'
                        || matches!(prev_nonws, b')' | b']')
                    {
                        j += 1; // past `{`; skip the whole block
                        let mut depth = 1usize;
                        while j < len && depth > 0 {
                            match bytes[j] {
                                b'\\' => { j += 2; }
                                b'"' | b'\'' => {
                                    let q = bytes[j]; j += 1;
                                    while j < len { if bytes[j] == b'\\' { j += 2; continue; } if bytes[j] == q { j += 1; break; } j += 1; }
                                }
                                b'{' => { depth += 1; j += 1; }
                                b'}' => { depth -= 1; j += 1; }
                                _ => { j += 1; }
                            }
                        }
                        continue;
                    }
                    // Flag if next char is not a space
                    if next != b' ' && next != b'\n' && next != 0 {
                        let pos = (line_start + j + 1) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space after `{` in hash literal.".into(),
                            range: TextRange::new(pos, pos),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Detect `}` not preceded by space and not empty `{}`
                if b == b'}' {
                    let prev = if j > 0 { bytes[j-1] } else { 0 };
                    // Skip empty braces already handled above
                    if prev == b'{' {
                        j += 1;
                        continue;
                    }
                    // Flag if prev char is not a space.
                    // Also skip `}` preceded by another `}` (consecutive block closes like `}}`).
                    if prev != b' ' && prev != 0 && prev != b'}' {
                        let pos = (line_start + j) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Missing space before `}` in hash literal.".into(),
                            range: TextRange::new(pos, pos),
                            severity: Severity::Warning,
                        });
                    }
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        // Insert a space at the flagged position
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: " ".into(),
            }],
            safety: FixSafety::Safe,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::LintContext;
    use std::path::Path;

    fn check(src: &str) -> Vec<String> {
        let ctx = LintContext::new(Path::new("test.rb"), src);
        SpaceInsideHashLiteralBraces.check_source(&ctx)
            .into_iter()
            .map(|d| d.message)
            .collect()
    }

    // Regression: CSS/HTML inside a heredoc contained `{margin: 0}` etc. which
    // was previously scanned as Ruby, producing false-positive hash-brace warnings.
    #[test]
    fn test_heredoc_body_not_scanned() {
        let src = concat!(
            "html = <<-HTML\n",
            "  <style>body {margin: 0; padding: 0;}</style>\n",
            "HTML\n",
        );
        assert!(check(src).is_empty(), "heredoc body must not be scanned for hash braces");
    }

    // Regression: squiggly heredoc (<<~) also skipped.
    #[test]
    fn test_squiggly_heredoc_body_not_scanned() {
        let src = concat!(
            "css = <<~CSS\n",
            "  .foo {color: red;}\n",
            "CSS\n",
        );
        assert!(check(src).is_empty(), "<<~ heredoc body must not be scanned");
    }

    // Sanity: genuine missing-space hash on real Ruby line still flagged.
    #[test]
    fn test_real_hash_missing_space_still_flagged() {
        let src = "h = {a: 1}\n";
        let diags = check(src);
        assert!(!diags.is_empty(), "missing space after {{ should still be flagged");
    }
}
