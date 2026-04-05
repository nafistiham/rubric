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

/// Build a scope-path string for each line by tracking class/module nesting.
/// Uses a two-stack approach: one for ALL block openers (to count `end`s),
/// one for class/module names (to build the scope path).
fn build_scope_paths<'a>(lines: &[&'a str]) -> Vec<String> {
    let mut result = Vec::with_capacity(lines.len());
    let mut class_stack: Vec<(usize, String)> = Vec::new(); // (nesting_depth, name)
    let mut nesting: usize = 0;

    for line in lines.iter() {
        // Record current scope path before processing this line
        let path: String = class_stack.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join("::");
        result.push(path);

        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let scan_end = trimmed.find('#').unwrap_or(trimmed.len());
        let code = &trimmed[..scan_end];

        // Detect class/module openers
        let is_class_or_module = code.starts_with("class ") || code.starts_with("module ");
        // Detect other block openers that consume an `end`
        let is_block_opener = is_class_or_module
            || code.starts_with("def ")
            || code.starts_with("begin")
            || code.starts_with("do ")
            || code.ends_with(" do")
            || code.contains(" do |")
            || code.contains(" do\n")
            || code.starts_with("if ")
            || code.starts_with("unless ")
            || code.starts_with("while ")
            || code.starts_with("until ")
            || code.starts_with("for ")
            || code.starts_with("case ")
            || code.starts_with("loop ")
            || code.contains("{|") // inline block with braces handled by matching `}`
            ;
        // Detect end — only standalone `end` or `end # comment`, not `end_with?`
        let is_end = code == "end"
            || code.starts_with("end ")
            || code.starts_with("end#");

        if is_class_or_module {
            // Extract the class/module name
            let rest = code.splitn(2, ' ').nth(1).unwrap_or("").trim();
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != ':')
                .unwrap_or(rest.len());
            let name = rest[..name_end].trim_matches(':').to_string();
            let name = if name.is_empty() { "<anon>".to_string() } else { name };
            class_stack.push((nesting, name));
            nesting += 1;
        } else if is_block_opener {
            nesting += 1;
        } else if is_end {
            if nesting > 0 {
                nesting -= 1;
                if let Some(&(depth, _)) = class_stack.last() {
                    if depth == nesting {
                        class_stack.pop();
                    }
                }
            }
        }
    }
    result
}

impl Rule for ConstantReassignment {
    fn name(&self) -> &'static str {
        "Lint/ConstantReassignment"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let scope_paths = build_scope_paths(&ctx.lines);

        // Map from (scope_path, constant_name) to list of (line_index, col_offset)
        let mut assignments: HashMap<(String, &str), Vec<(usize, usize)>> = HashMap::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            if let Some((name, col_offset)) = extract_constant_assignment(scan_slice) {
                let key = (scope_paths[i].clone(), name);
                assignments.entry(key).or_default().push((i, col_offset));
            }
        }

        // Collect diagnostics for all assignments after the first within the same scope
        let mut diags = Vec::new();
        for ((_, name), occurrences) in &assignments {
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
