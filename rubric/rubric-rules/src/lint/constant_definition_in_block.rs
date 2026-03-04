use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

/// Returns true when `trimmed` contains the `do` keyword as a block opener.
/// The `do` must be preceded by a space (or be the whole line) and followed
/// by a space, `|`, or end of string — so that words like `doorkeeper` or
/// `domain` are not mistaken for `do`.
fn line_opens_do_block(trimmed: &str) -> bool {
    // Strip inline comment so `foo # do something` is not counted.
    let code = strip_inline_comment(trimmed);
    let code = code.trim_end();

    if code == "do" {
        return true;
    }

    // Search for ` do` where `do` is followed by space, `|`, or end-of-string.
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut j = 0;
    while j + 2 <= len {
        if bytes[j] == b' ' && j + 2 < len && bytes[j + 1] == b'd' && bytes[j + 2] == b'o' {
            // Check the character after `do`
            let after = if j + 3 < len { bytes[j + 3] } else { 0 };
            if after == 0 || after == b' ' || after == b'|' {
                return true;
            }
        }
        j += 1;
    }
    false
}

/// Strips an inline comment from a code line (naive: finds first unquoted `#`).
fn strip_inline_comment(s: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double => return &s[..i],
            _ => {}
        }
        i += 1;
    }
    s
}

/// Returns the heredoc delimiter if the line opens a heredoc (`<<DELIM` or `<<~DELIM`).
/// We extract the delimiter identifier so we can detect the closing line.
fn heredoc_delimiter(trimmed: &str) -> Option<String> {
    // Find `<<` or `<<~` in the line.
    let mut i = 0;
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    while i + 1 < len {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            let mut j = i + 2;
            // Optional `~` or `-` indent modifier.
            if j < len && (bytes[j] == b'~' || bytes[j] == b'-') {
                j += 1;
            }
            // Optional quotes around delimiter.
            let quoted = j < len && (bytes[j] == b'\'' || bytes[j] == b'"' || bytes[j] == b'`');
            if quoted {
                j += 1;
            }
            // Collect identifier characters.
            let start = j;
            while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            if j > start {
                let delim = &trimmed[start..j];
                if !delim.is_empty() {
                    return Some(delim.to_string());
                }
            }
        }
        i += 1;
    }
    None
}

pub struct ConstantDefinitionInBlock;

impl Rule for ConstantDefinitionInBlock {
    fn name(&self) -> &'static str {
        "Lint/ConstantDefinitionInBlock"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Track block depth (do...end blocks).
        let mut block_depth = 0usize;
        // Track active heredoc closing delimiter; while set, skip body lines.
        let mut heredoc_end: Option<String> = None;

        for i in 0..n {
            let line = &lines[i];
            let trimmed = line.trim_start();

            // Handle heredoc body: skip until the closing delimiter line.
            if let Some(ref delim) = heredoc_end.clone() {
                if trimmed.trim_end() == delim.as_str() {
                    heredoc_end = None;
                }
                // Skip all lines inside the heredoc body (including the closing line).
                continue;
            }

            if trimmed.starts_with('#') {
                continue;
            }

            // Detect heredoc opening on this line; body starts on the NEXT line.
            if let Some(delim) = heredoc_delimiter(trimmed) {
                heredoc_end = Some(delim);
                // Still process the current line for block-depth changes below.
            }

            // Determine if this line opens a `do...end` block.
            let opens_block = line_opens_do_block(trimmed);
            // Determine if this line also starts a constant assignment
            // (CONST = ... do |block|). In that case the constant is the owner
            // of the block, not defined inside one — not a violation.
            let is_const_block_assignment = opens_block && is_constant_assignment(trimmed);

            if opens_block {
                block_depth += 1;
            }

            if trimmed == "end" && block_depth > 0 {
                block_depth -= 1;
            }

            // Inside a block, detect constant assignment — but only if the
            // constant is not the one that owns the block on this very line.
            if block_depth > 0 && !is_const_block_assignment {
                let t = trimmed;
                if !t.is_empty() {
                    let first = t.chars().next().unwrap_or(' ');
                    if first.is_ascii_uppercase() {
                        // Check if it looks like a constant assignment: CONST_NAME = value
                        if let Some(eq_pos) = t.find(" = ").or_else(|| t.find("=")) {
                            let lhs = &t[..eq_pos];
                            // LHS should be all uppercase/digits/underscores.
                            if lhs.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                                let indent = line.len() - trimmed.len();
                                let line_start = ctx.line_start_offsets[i] as usize;
                                let pos = (line_start + indent) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Do not define constant `{}` inside a block.",
                                        lhs
                                    ),
                                    range: TextRange::new(pos, pos + lhs.len() as u32),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }
        }

        diags
    }
}

/// Returns true when `trimmed` starts with an all-uppercase constant name
/// followed by ` = ` (i.e., this is a constant assignment line).
fn is_constant_assignment(trimmed: &str) -> bool {
    let first = trimmed.chars().next().unwrap_or(' ');
    if !first.is_ascii_uppercase() {
        return false;
    }
    if let Some(eq_pos) = trimmed.find(" = ") {
        let lhs = &trimmed[..eq_pos];
        lhs.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    } else {
        false
    }
}
