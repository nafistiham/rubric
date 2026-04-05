use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NumericLiterals;

/// Minimum number of digits for a literal to require underscore separators.
const MIN_DIGITS: usize = 5;

/// Returns the real comment start position on a line, skipping `#` inside strings.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
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
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        i += 1;
    }
    None
}

/// Returns true if the byte at `pos` in `bytes` is inside a string literal.
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
            None if bytes[i] == b'#' => {
                return false;
            }
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Returns true if the character at `pos` in `bytes` is preceded by a
/// special numeric prefix (`0x`, `0b`, `0o`, or `0X`, `0B`, `0O`).
fn has_special_prefix(bytes: &[u8], pos: usize) -> bool {
    if pos < 2 {
        return false;
    }
    if bytes[pos - 2] != b'0' {
        return false;
    }
    matches!(bytes[pos - 1], b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
}

impl Rule for NumericLiterals {
    fn name(&self) -> &'static str {
        "Style/NumericLiterals"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Limit scan to before any inline comment
            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_bytes = line[..scan_end].as_bytes();
            let line_start = ctx.line_start_offsets[i] as usize;

            let mut pos = 0;
            let mut in_string: Option<u8> = None; // inside "..." or '...'
            let mut in_regex = false;              // inside /regex/
            while pos < scan_bytes.len() {
                let b = scan_bytes[pos];

                // Track string/regex context
                if let Some(delim) = in_string {
                    if b == b'\\' { pos += 2; continue; }
                    if b == delim { in_string = None; }
                    pos += 1;
                    continue;
                }
                if in_regex {
                    if b == b'\\' { pos += 2; continue; }
                    if b == b'/' { in_regex = false; }
                    pos += 1;
                    continue;
                }

                // Detect string/regex openers
                match b {
                    b'"' | b'\'' => { in_string = Some(b); pos += 1; continue; }
                    b'/' => {
                        // Regex if preceded by =, (, ,, [, space/start, !, |, &, ?, :
                        let prev_nonws = scan_bytes[..pos].iter().rposition(|&c| c != b' ' && c != b'\t')
                            .map(|p| scan_bytes[p]);
                        if matches!(prev_nonws, None
                            | Some(b'=') | Some(b'(') | Some(b',') | Some(b'[')
                            | Some(b'!') | Some(b'|') | Some(b'&') | Some(b'?')
                            | Some(b':') | Some(b';') | Some(b'{')) {
                            in_regex = true;
                            pos += 1;
                            continue;
                        }
                    }
                    _ => {}
                }

                if b.is_ascii_digit() {
                    // Collect the full numeric token (digits + underscores + dot)
                    let token_start = pos;
                    while pos < scan_bytes.len()
                        && (scan_bytes[pos].is_ascii_digit()
                            || scan_bytes[pos] == b'_'
                            || scan_bytes[pos] == b'.')
                    {
                        // Stop at decimal point if what follows is not a digit
                        // (i.e., method call like `1.times`)
                        if scan_bytes[pos] == b'.' {
                            let next = pos + 1;
                            if next >= scan_bytes.len() || !scan_bytes[next].is_ascii_digit() {
                                break;
                            }
                        }
                        pos += 1;
                    }

                    // Extract the token
                    let token = &line[..scan_end][token_start..pos];

                    // Skip tokens that are part of hex/binary/octal (preceded by 0x/0b/0o)
                    if has_special_prefix(scan_bytes, token_start) {
                        continue;
                    }

                    // Skip numbers that are part of an identifier, symbol name, or
                    // method name (e.g. `:index_20180106`, `func_name123`).
                    if token_start > 0 {
                        let prev = scan_bytes[token_start - 1];
                        if prev.is_ascii_alphanumeric() || prev == b'_' {
                            continue;
                        }
                    }

                    // Count only digit characters (no underscores, no dots)
                    let digit_count = token.chars().filter(|c| c.is_ascii_digit()).count();
                    let has_underscore = token.contains('_');

                    if digit_count >= MIN_DIGITS && !has_underscore {
                        let start = (line_start + token_start) as u32;
                        let end = (line_start + pos) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: "Use underscores(_) as numeric separators and add them every 3 digits.".into(),
                            range: TextRange::new(start, end),
                            severity: Severity::Warning,
                        });
                    }

                    continue;
                }

                pos += 1;
            }
        }

        diags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubric_core::LintContext;
    use std::path::Path;

    #[test]
    fn detects_large_integer_without_underscores() {
        let src = "THRESHOLD = 10000\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(!diags.is_empty(), "expected violation for 10000");
        assert!(diags.iter().all(|d| d.rule == "Style/NumericLiterals"));
    }

    #[test]
    fn detects_million_without_underscores() {
        let src = "population = 1000000\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(!diags.is_empty(), "expected violation for 1000000");
    }

    #[test]
    fn no_violation_for_underscored_literal() {
        let src = "THRESHOLD = 10_000\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_hex_literal() {
        let src = "hex_val = 0xFF00FF\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation for hex; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_binary_literal() {
        let src = "binary = 0b10101010\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation for binary; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_small_number() {
        let src = "n = 9999\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation for 9999; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_number_in_comment() {
        let src = "x = 1 # total is 10000 items\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation for number in comment; got: {:?}", diags);
    }

    #[test]
    fn no_violation_for_number_in_string() {
        let src = "msg = \"value 10000\"\n";
        let ctx = LintContext::new(Path::new("test.rb"), src);
        let diags = NumericLiterals.check_source(&ctx);
        assert!(diags.is_empty(), "expected no violation for number in string; got: {:?}", diags);
    }
}
