use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct RescuedExceptionsVariableName;

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

/// Extracts the rescue variable name from a line containing `=> <name>`.
/// Returns `None` if the pattern is not found.
fn extract_rescue_variable(scan_slice: &str) -> Option<&str> {
    // Find `=>`
    let arrow_pos = scan_slice.find("=>")?;
    // The rest after `=>`
    let after_arrow = scan_slice[arrow_pos + 2..].trim();
    // The variable name is the first word token
    let name: &str = after_arrow
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

impl Rule for RescuedExceptionsVariableName {
    fn name(&self) -> &'static str {
        "Naming/RescuedExceptionsVariableName"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip full comment lines
            if trimmed.starts_with('#') {
                continue;
            }

            // Only process lines that contain `rescue`
            if !trimmed.starts_with("rescue") {
                continue;
            }

            let scan_end = comment_start(line).unwrap_or(line.len());
            let scan_slice = &line[..scan_end];

            // Must contain `=>` to have a named rescue variable
            if !scan_slice.contains("=>") {
                continue;
            }

            let var_name = match extract_rescue_variable(scan_slice) {
                Some(name) => name,
                None => continue,
            };

            // The preferred name is `e` — flag anything else
            if var_name == "e" {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            // Find the position of the variable name after `=>`
            let arrow_pos = scan_slice.find("=>").unwrap();
            let after_arrow_trimmed_offset = scan_slice[arrow_pos + 2..]
                .len()
                - scan_slice[arrow_pos + 2..].trim_start().len();
            let var_start = arrow_pos + 2 + after_arrow_trimmed_offset;
            let start = (line_start + var_start) as u32;
            let end = start + var_name.len() as u32;

            diags.push(Diagnostic {
                rule: self.name(),
                message: format!(
                    "Use e as the rescued exceptions variable name instead of {}.",
                    var_name
                ),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diags
    }
}
