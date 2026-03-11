use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct SpaceAroundKeyword;

// Keywords that must be followed by a space (not immediately by `(`)
const KEYWORDS: &[&str] = &["if", "unless", "while", "until", "and", "or", "not", "in", "do"];

impl Rule for SpaceAroundKeyword {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundKeyword"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let bytes = line.as_bytes();
            let len = bytes.len();
            let line_start = ctx.line_start_offsets[i];

            // Single-pass scan with string-literal tracking
            let mut in_string: Option<u8> = None;
            let mut j = 0;
            while j < len {
                let b = bytes[j];

                // ── String state ────────────────────────────────────────────
                match in_string {
                    Some(_) if b == b'\\' => { j += 2; continue; }
                    Some(delim) if b == delim => { in_string = None; j += 1; continue; }
                    Some(_) => { j += 1; continue; }
                    None if b == b'"' || b == b'\'' || b == b'`' => {
                        in_string = Some(b); j += 1; continue;
                    }
                    None if b == b'#' => break, // inline comment — stop
                    None => {}
                }

                // ── Check each keyword at position j ─────────────────────
                for kw in KEYWORDS {
                    let kw_bytes = kw.as_bytes();
                    let kw_len = kw_bytes.len();
                    if j + kw_len > len { continue; }
                    if &bytes[j..j + kw_len] != kw_bytes { continue; }

                    // Word boundary before: no alphanumeric/underscore/dot preceding
                    let before_ok = j == 0 || {
                        let pb = bytes[j - 1];
                        !pb.is_ascii_alphanumeric() && pb != b'_' && pb != b'.'
                    };
                    // Skip if preceded by `def ` — keyword used as a method name
                    // (e.g. `def not(...)`, `def and(...)`, `def in(...)`)
                    let preceded_by_def = {
                        let mut p = j;
                        while p > 0 && (bytes[p - 1] == b' ' || bytes[p - 1] == b'\t') { p -= 1; }
                        p >= 3 && &bytes[p - 3..p] == b"def"
                            && (p == 3 || bytes[p - 4] == b' ' || bytes[p - 4] == b'\t')
                    };
                    // Keyword immediately followed by `(`
                    let after_pos = j + kw_len;
                    if before_ok && !preceded_by_def && after_pos < len && bytes[after_pos] == b'(' {
                        let kw_start = line_start + j as u32;
                        let kw_end = kw_start + kw_len as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!("Keyword `{}` should be followed by a space.", kw),
                            range: TextRange::new(kw_start, kw_end),
                            severity: Severity::Warning,
                        });
                    }
                }

                j += 1;
            }
        }

        diags
    }
}
