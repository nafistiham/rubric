use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct UnderscorePrefixedVariableName;

/// Returns true if column `col` in `line` is inside a string or regex literal.
/// Only scans the current line, so cross-line state corruption cannot occur.
fn in_string_or_regex(line: &[u8], col: usize) -> bool {
    let mut in_delim: Option<u8> = None;
    let mut i = 0;
    while i < col && i < line.len() {
        let b = line[i];
        if let Some(d) = in_delim {
            if b == b'\\' { i += 2; continue; }
            if b == d { in_delim = None; }
        } else {
            match b {
                b'"' | b'\'' => { in_delim = Some(b); }
                b'/' => {
                    // Distinguish regex `/` from division `/`.
                    // Same heuristic as semicolon.rs: `/` is a regex start when
                    // preceded by whitespace, `(`, `,`, `=`, operator chars, or
                    // an alphanumeric/`_` (method name like `match`, `not_to`).
                    let prev = line[..i].iter().rposition(|&b| b != b' ' && b != b'\t')
                        .map(|p| line[p]);
                    let is_regex_ctx = matches!(prev, None
                        | Some(b'(') | Some(b',') | Some(b'=') | Some(b'!')
                        | Some(b'|') | Some(b'&') | Some(b'?') | Some(b':')
                        | Some(b'[') | Some(b'{'))
                        || prev.map_or(false, |c| c.is_ascii_alphabetic() || c == b'_');
                    if is_regex_ctx {
                        in_delim = Some(b'/');
                    }
                }
                b'#' => break, // comment — nothing after counts
                _ => {}
            }
        }
        i += 1;
    }
    in_delim.is_some()
}

impl Rule for UnderscorePrefixedVariableName {
    fn name(&self) -> &'static str {
        "Lint/UnderscorePrefixedVariableName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let src = ctx.source;
        let lines = &ctx.lines;
        let n = lines.len();

        // Collect `_var` assignments
        let mut underscore_vars: Vec<(String, usize)> = Vec::new(); // (name, line_idx)

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();
            if t.starts_with('#') { continue; }

            if let Some(eq_pos) = t.find(" = ") {
                let lhs = t[..eq_pos].trim();
                if lhs.starts_with('_') && lhs.len() > 1
                    && lhs.chars().nth(1).map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
                    underscore_vars.push((lhs.to_string(), i));
                }
            }
        }

        // Check if each `_var` is used elsewhere in the source
        for (var, assign_line) in &underscore_vars {
            let vb = var.as_bytes();
            let bb = src.as_bytes();
            let src_len = bb.len();
            let vn = vb.len();
            let mut used = false;

            let mut pos = 0;
            while pos + vn <= src_len {
                if &bb[pos..pos + vn] == vb {
                    let before_ok = pos == 0 || !bb[pos - 1].is_ascii_alphanumeric() && bb[pos - 1] != b'_';
                    let after_ok = pos + vn >= src_len || !bb[pos + vn].is_ascii_alphanumeric() && bb[pos + vn] != b'_';
                    if before_ok && after_ok {
                        // Find which line this occurrence is on.
                        let line_of_occurrence = ctx.line_start_offsets.partition_point(|&o| o as usize <= pos).saturating_sub(1);
                        if line_of_occurrence != *assign_line {
                            let line_start = ctx.line_start_offsets[line_of_occurrence] as usize;
                            let pos_in_line = pos - line_start;
                            let line_bytes = lines[line_of_occurrence].as_bytes();

                            // Skip occurrences that are themselves LHS assignments
                            // of the same variable (e.g. `_tag = ...` in another block).
                            // Only count as a use when the occurrence is a read.
                            let is_lhs_assignment = {
                                let after_var = &line_bytes[pos_in_line + vn..];
                                let mut k = 0;
                                while k < after_var.len() && after_var[k] == b' ' { k += 1; }
                                k < after_var.len()
                                    && after_var[k] == b'='
                                    && (k + 1 >= after_var.len()
                                        || (after_var[k + 1] != b'=' && after_var[k + 1] != b'>'))
                            };
                            // Skip occurrences inside string/regex literals — their
                            // content is not a variable reference
                            // (e.g. `"show-source _c.new.method"` or `/_version/`).
                            if !is_lhs_assignment && !in_string_or_regex(line_bytes, pos_in_line) {
                                used = true;
                                break;
                            }
                        }
                    }
                }
                pos += 1;
            }

            if used {
                let indent = lines[*assign_line].len() - lines[*assign_line].trim_start().len();
                let line_start = ctx.line_start_offsets[*assign_line] as usize;
                let pos_out = (line_start + indent) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!(
                        "Variable `{}` is prefixed with `_` but is actually used.",
                        var
                    ),
                    range: TextRange::new(pos_out, pos_out + var.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }

        diags
    }
}
