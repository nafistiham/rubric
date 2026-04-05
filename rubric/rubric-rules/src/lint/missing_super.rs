use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MissingSuper;

fn in_string_context(bytes: &[u8], pos: usize) -> bool {
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < pos {
        match in_str {
            Some(_) if bytes[i] == b'\\' => { i += 2; continue; }
            Some(d) if bytes[i] == d => { in_str = None; }
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => { in_str = Some(bytes[i]); }
            None if bytes[i] == b'#' => break,
            None => {}
        }
        i += 1;
    }
    in_str.is_some()
}

/// Check if `super` appears as a real keyword (not inside a string or comment) in the given line.
fn line_has_super(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Check if current position starts `super`
        if bytes[i..].starts_with(b"super") {
            let after = i + 5;
            // Must be word-boundary after
            let after_ok = after >= bytes.len()
                || !bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_';
            // Must be word-boundary before
            let before_ok = i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            if before_ok && after_ok && !in_string_context(bytes, i) {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Count the net change in nesting depth from def/do/class/module openers vs `end` closers.
/// This is a simple heuristic to find the matching `end` of `def initialize`.
fn net_depth_change(line: &str) -> i32 {
    let trimmed = line.trim();
    // Skip comments
    if trimmed.starts_with('#') {
        return 0;
    }
    let opens: i32 = count_openers(trimmed) as i32;
    let closes: i32 = count_closers(trimmed) as i32;
    opens - closes
}

fn count_openers(trimmed: &str) -> usize {
    let mut count = 0;
    // def, class, module, do, begin, if/unless/while/until/for/case at start of statement
    for kw in &["def ", "class ", "module ", " do", "\tdo", "do\n", "begin", "if ", "unless ", "while ", "until ", "for ", "case "] {
        if trimmed.contains(kw) {
            count += 1;
        }
    }
    // Also check: trimmed == "begin" or ends with " do" or "do"
    if trimmed == "begin" || trimmed.ends_with(" do") || trimmed.ends_with("\tdo") {
        count += 1;
    }
    count
}

fn count_closers(trimmed: &str) -> usize {
    // Count `end` occurrences (simple: word boundaries)
    let mut count = 0;
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"end") {
            let after = i + 3;
            let after_ok = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_';
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            if before_ok && after_ok {
                count += 1;
            }
        }
        i += 1;
    }
    count
}

/// Scan backwards from `def_line` to find the immediately enclosing class
/// declaration. Returns true if that class has an explicit parent (`< Parent`).
/// Classes without `<` inherit from Object implicitly — rubocop does not flag
/// missing super for those.
fn enclosing_class_has_parent(lines: &[&str], def_line: usize) -> bool {
    let mut depth = 0i32; // positive = inside nested blocks we need to skip
    let mut j = def_line as i64 - 1;
    while j >= 0 {
        let line = lines[j as usize].trim();
        if line.starts_with('#') { j -= 1; continue; }
        let closes = count_closers(line) as i32;
        let opens_n = count_openers(line) as i32;
        depth += closes;
        // For each opener on this line, check if depth reaches 0
        for _ in 0..opens_n {
            if depth <= 0 {
                // This is the opener of the immediately enclosing block
                // Check if it's a class with explicit parent
                return line.contains(" < ") && (line.starts_with("class ") || line.contains(" class "));
            }
            depth -= 1;
        }
        j -= 1;
    }
    false
}

impl Rule for MissingSuper {
    fn name(&self) -> &'static str {
        "Lint/MissingSuper"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines: Vec<&str> = ctx.lines.iter().map(|s| s.as_ref()).collect();
        let n = lines.len();
        let mut i = 0;

        while i < n {
            let trimmed = lines[i].trim();

            // Look for `def initialize` lines
            let is_init = trimmed == "def initialize"
                || trimmed.starts_with("def initialize(")
                || trimmed.starts_with("def initialize ")
                || trimmed.starts_with("def initialize\t");

            if !is_init || trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Only flag if inside a class with explicit parent (`class Foo < Bar`).
            // Classes without explicit parents inherit from Object — rubocop
            // does not require super in their initialize methods.
            if !enclosing_class_has_parent(&lines, i) {
                i += 1;
                continue;
            }

            // One-liner: `def initialize; ...; end` — check same line
            if trimmed.contains("; end") || trimmed.ends_with(" end") {
                if !line_has_super(trimmed) {
                    let line_start = ctx.line_start_offsets[i];
                    let def_col = lines[i].len() - lines[i].trim_start().len();
                    let start = line_start + def_col as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Call `super` in `initialize`, or explicitly define an empty method without `super`.".into(),
                        range: TextRange::new(start, start + 3),
                        severity: Severity::Warning,
                    });
                }
                i += 1;
                continue;
            }

            // Multi-line: scan forward to matching `end`
            let def_line = i;
            let mut depth = 1i32;
            let mut found_super = false;
            let mut j = i + 1;

            while j < n && depth > 0 {
                let body_line = lines[j].trim();
                // Check for super at any nesting level inside initialize
                if line_has_super(lines[j]) {
                    found_super = true;
                }
                // Update depth
                depth += net_depth_change(body_line);
                j += 1;
            }

            if !found_super {
                let line_start = ctx.line_start_offsets[def_line];
                let def_col = lines[def_line].len() - lines[def_line].trim_start().len();
                let start = line_start + def_col as u32;
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Call `super` in `initialize`, or explicitly define an empty method without `super`.".into(),
                    range: TextRange::new(start, start + 3),
                    severity: Severity::Warning,
                });
            }

            i = j;
        }

        diags
    }
}
