use rubric_core::{Diagnostic, Fix, LintContext, Rule, Severity, TextRange};

pub struct HashSyntax;

/// Returns true if `line` contains at least one `key => value` pair where the key
/// is not a plain symbol literal (i.e., does not start with `:`).  This indicates
/// a hash with mixed key styles; RuboCop's `ruby19_no_mixed_keys` style leaves such
/// hashes alone, so we must not flag the symbol keys in them either.
fn has_non_symbol_rocket_key(line: &str) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
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

        // Look for `=>` outside strings
        if b == b'=' && j + 1 < len && bytes[j + 1] == b'>' {
            // Scan backwards to find the key token.
            // Skip spaces before `=>`
            let mut k = j;
            while k > 0 && bytes[k - 1] == b' ' {
                k -= 1;
            }
            // The character just before the `=>` (after spaces) tells us the key type.
            // If the key ends with an alphanumeric/`_`/`]`/`)` character, scan back
            // to find the start of the key token.
            if k > 0 {
                let key_end = k - 1; // last byte of key token
                // Walk back over the key token
                let mut key_start = key_end;
                while key_start > 0
                    && (bytes[key_start - 1].is_ascii_alphanumeric()
                        || bytes[key_start - 1] == b'_'
                        || bytes[key_start - 1] == b'?'
                        || bytes[key_start - 1] == b'!')
                {
                    key_start -= 1;
                }
                // If the key token is preceded by `::` it's a constant qualifier — the
                // real key starts further back, skip this heuristic for that position.
                // We only care whether the key starts with `:` (symbol) or not.
                let key_byte = if key_start > 0 { bytes[key_start - 1] } else { 0 };
                // key_byte is the character immediately before the word token.
                // If it is `:` and not `::`, the key is a symbol literal.
                let preceded_by_colon = key_byte == b':';
                let preceded_by_double_colon =
                    preceded_by_colon && key_start >= 2 && bytes[key_start - 2] == b':';

                if !preceded_by_colon || preceded_by_double_colon {
                    // The key is NOT a plain `:symbol` — it is a string, constant,
                    // variable, ivar, or namespace-qualified constant.
                    return true;
                }
            }
            j += 2;
            continue;
        }

        j += 1;
    }
    false
}

impl Rule for HashSyntax {
    fn name(&self) -> &'static str {
        "Style/HashSyntax"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let len = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            // Skip lines that contain a mix of symbol and non-symbol rocket keys.
            // RuboCop's `ruby19_no_mixed_keys` enforcement leaves such hashes alone.
            if has_non_symbol_rocket_key(line) {
                continue;
            }

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

                // Look for `:symbol =>` pattern: starts with `:` followed by word chars then ` =>`
                // Skip `::` namespace separators — the second `:` of `::` is not a symbol literal.
                // E.g. `ActiveRecord::RecordInvalid => e` must not be flagged.
                if b == b':' {
                    if j > 0 && bytes[j - 1] == b':' {
                        j += 1;
                        continue;
                    }
                    // Read the symbol name (word chars)
                    let mut k = j + 1;
                    while k < len && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_') {
                        k += 1;
                    }

                    // Check if symbol name is non-empty and followed by ` =>`
                    if k > j + 1 {
                        // Skip spaces
                        let mut m = k;
                        while m < len && bytes[m] == b' ' {
                            m += 1;
                        }
                        if m + 1 < len && bytes[m] == b'=' && bytes[m+1] == b'>' {
                            let pos = (line_start + j) as u32;
                            let end = (line_start + m + 2) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use the new hash syntax (`key: value`) instead of `{:key => value}`.".into(),
                                range: TextRange::new(pos, end),
                                severity: Severity::Warning,
                            });
                            j = m + 2;
                            continue;
                        }
                    }
                }

                j += 1;
            }
        }

        diags
    }

    fn fix(&self, _diag: &Diagnostic) -> Option<Fix> {
        None // TODO: implement :sym => val -> sym: val transformation
    }
}
