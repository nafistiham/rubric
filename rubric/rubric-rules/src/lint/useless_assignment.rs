use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashMap;

pub struct UselessAssignment;

impl Rule for UselessAssignment {
    fn name(&self) -> &'static str {
        "Lint/UselessAssignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Process each `def..end` block independently
        let mut i = 0;
        while i < n {
            let trimmed = lines[i].trim();
            if !trimmed.starts_with("def ") && trimmed != "def" {
                i += 1;
                continue;
            }

            // Find the matching `end`
            let def_start = i;
            let mut depth = 1i32;
            let mut j = i + 1;
            while j < n && depth > 0 {
                let t = lines[j].trim();
                if t.starts_with("def ") || t.starts_with("if ") || t.starts_with("unless ")
                    || t.starts_with("do ") || t == "do" || t.starts_with("begin")
                    || t.starts_with("case ")
                    || t.starts_with("while ") || t == "while"
                    || t.starts_with("until ") || t == "until"
                    || t.starts_with("for ")
                    || t == "loop" || t.ends_with(" loop")
                {
                    depth += 1;
                }
                if t == "end" {
                    depth -= 1;
                }
                j += 1;
            }
            let def_end = j; // exclusive

            // Within def_start..def_end, collect assignments and usages
            // assignment: `<var> = ` at start of expression
            // usage: any occurrence of `<var>` in the remaining lines

            // Map: var_name -> (line_idx, col_in_line)
            let mut assignments: HashMap<String, (usize, usize)> = HashMap::new();

            for k in (def_start + 1)..def_end.min(n) {
                let line = &lines[k];
                let line_bytes = line.as_bytes();
                let line_len = line_bytes.len();
                let mut pos = 0;
                let mut in_string: Option<u8> = None;

                while pos < line_len {
                    let b = line_bytes[pos];
                    match in_string {
                        Some(_) if b == b'\\' => { pos += 2; continue; }
                        Some(delim) if b == delim => { in_string = None; pos += 1; continue; }
                        Some(_) => { pos += 1; continue; }
                        None if b == b'"' || b == b'\'' => { in_string = Some(b); pos += 1; continue; }
                        None if b == b'#' => break,
                        None => {}
                    }

                    // Look for `<word> = ` assignment pattern
                    if b.is_ascii_lowercase() || b == b'_' {
                        let word_start = pos;
                        while pos < line_len && (line_bytes[pos].is_ascii_alphanumeric() || line_bytes[pos] == b'_') {
                            pos += 1;
                        }
                        let word = std::str::from_utf8(&line_bytes[word_start..pos]).unwrap_or("");

                        // Skip keywords
                        if matches!(word, "def" | "end" | "if" | "unless" | "else" | "do" | "return" | "begin" | "case" | "while" | "until") {
                            continue;
                        }

                        // Skip words starting with `_`
                        if word.starts_with('_') {
                            continue;
                        }

                        // Skip whitespace and check for `=` not followed by `=` or `>`
                        let mut eq_pos = pos;
                        while eq_pos < line_len && line_bytes[eq_pos] == b' ' {
                            eq_pos += 1;
                        }

                        if eq_pos < line_len && line_bytes[eq_pos] == b'=' {
                            let after_eq = if eq_pos + 1 < line_len { line_bytes[eq_pos+1] } else { 0 };
                            if after_eq != b'=' && after_eq != b'>' {
                                // This is an assignment
                                // Only record if it's at the beginning of a meaningful expression
                                // (skip if word_start is not at the beginning of a statement)
                                let indent_end = line.len() - line.trim_start().len();
                                if word_start == indent_end {
                                    assignments.insert(word.to_string(), (k, word_start));
                                }
                            }
                        }
                        continue;
                    }

                    pos += 1;
                }
            }

            // Now check which assigned variables are never read
            // A variable is read if it appears in the block NOT as a write (LHS of assignment)
            'outer: for (var, (assign_line, assign_col)) in &assignments {
                // Check all lines in the block for a read of `var`
                for k in (def_start + 1)..def_end.min(n) {
                    let line = &lines[k];
                    let line_bytes = line.as_bytes();
                    let line_len = line_bytes.len();
                    let mut pos = 0;
                    let mut in_string: Option<u8> = None;

                    while pos < line_len {
                        let b = line_bytes[pos];
                        match in_string {
                            Some(_) if b == b'\\' => { pos += 2; continue; }
                            Some(delim) if b == delim => { in_string = None; pos += 1; continue; }
                            Some(_) => { pos += 1; continue; }
                            None if b == b'"' || b == b'\'' => { in_string = Some(b); pos += 1; continue; }
                            None if b == b'#' => break,
                            None => {}
                        }

                        // Check if `var` appears at this position
                        let vb = var.as_bytes();
                        if pos + vb.len() <= line_len && &line_bytes[pos..pos+vb.len()] == vb {
                            // Check word boundary (not part of a larger identifier)
                            let before_ok = pos == 0 || !line_bytes[pos-1].is_ascii_alphanumeric() && line_bytes[pos-1] != b'_';
                            let after_ok = pos + vb.len() >= line_len || !line_bytes[pos+vb.len()].is_ascii_alphanumeric() && line_bytes[pos+vb.len()] != b'_';

                            if before_ok && after_ok {
                                // Check if this is the assignment itself (LHS)
                                let is_assignment_lhs = k == *assign_line && pos == *assign_col;
                                if !is_assignment_lhs {
                                    // This is a read — variable is used, skip
                                    continue 'outer;
                                }
                            }
                        }
                        pos += 1;
                    }
                }

                // Variable was never read — report useless assignment
                let line_start = ctx.line_start_offsets[*assign_line] as usize;
                let pos = (line_start + assign_col) as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Useless assignment to variable `{}`.", var),
                    range: TextRange::new(pos, pos + var.len() as u32),
                    severity: Severity::Warning,
                });
            }

            i = def_end;
        }

        diags
    }
}
