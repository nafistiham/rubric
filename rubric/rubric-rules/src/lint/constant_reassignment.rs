use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};
use std::collections::HashMap;

pub struct ConstantReassignment;

/// Returns the byte index of a real `#` comment character in `line`, skipping
/// `#` inside string literals or interpolations.
fn comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_str: Option<u8> = None;
    let mut interp_depth: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match in_str {
            Some(_) if bytes[i] == b'\\' => {
                i += 2;
                continue;
            }
            Some(b'"') if bytes[i..].starts_with(b"#{") => {
                interp_depth += 1;
                i += 2;
                continue;
            }
            Some(d) if bytes[i] == d && interp_depth == 0 => {
                in_str = None;
            }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => {
                in_str = Some(bytes[i]);
            }
            None if bytes[i] == b'#' => return Some(i),
            None => {}
        }
        if interp_depth > 0 && bytes[i] == b'}' {
            interp_depth -= 1;
        }
        i += 1;
    }
    None
}

/// If the line is a constant assignment (`CONST = value`, not `==`), returns
/// the constant name and its starting byte position within the line.
/// Returns `None` if this is not a constant assignment line.
fn extract_constant_assignment<'a>(scan_slice: &'a str) -> Option<(&'a str, usize)> {
    let trimmed = scan_slice.trim_start();
    let leading_spaces = scan_slice.len() - trimmed.len();

    // First character must be uppercase ASCII — constants start with uppercase
    let first_char = trimmed.chars().next()?;
    if !first_char.is_ascii_uppercase() {
        return None;
    }

    // Extract the constant name: uppercase letters, digits, underscores
    let name_end = trimmed
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(trimmed.len());
    let name = &trimmed[..name_end];

    // The name must be entirely uppercase (or uppercase + digits/underscores)
    if !name.chars().next()?.is_ascii_uppercase() {
        return None;
    }

    // After the name, expect optional whitespace then `=` then NOT `=`
    let after_name = trimmed[name_end..].trim_start();
    if !after_name.starts_with('=') {
        return None;
    }
    let after_eq = &after_name[1..];
    // Must not be `==` (comparison)
    if after_eq.starts_with('=') {
        return None;
    }

    Some((name, leading_spaces))
}

impl Rule for ConstantReassignment {
    fn name(&self) -> &'static str {
        "Lint/ConstantReassignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        // Map from constant name to list of (line_index, start_byte_offset_in_line)
        let mut assignments: HashMap<&str, Vec<(usize, usize)>> = HashMap::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            if let Some((name, col_offset)) = extract_constant_assignment(scan_slice) {
                assignments.entry(name).or_default().push((i, col_offset));
            }
        }

        // Collect diagnostics for all assignments after the first
        let mut diags = Vec::new();
        for (name, occurrences) in &assignments {
            // Flag every occurrence after the first
            for &(line_idx, col_offset) in occurrences.iter().skip(1) {
                let line_start = ctx.line_start_offsets[line_idx] as usize;
                let start = (line_start + col_offset) as u32;
                let end = start + name.len() as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: format!("Constant {} is being reassigned.", name),
                    range: TextRange::new(start, end),
                    severity: Severity::Warning,
                });
            }
        }

        // Sort by start offset so output is deterministic
        diags.sort_by_key(|d| d.range.start);

        diags
    }
}
