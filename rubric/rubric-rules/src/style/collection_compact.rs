use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct CollectionCompact;

/// Check whether a block body (the content between `{` and `}`) contains
/// a nil-filtering pattern:
/// - `.nil?` with preceding `!` (for `select`/`filter`)
/// - `.nil?` without `!` (for `reject`)
/// - `!= nil` (for `select`/`filter`)
/// - `== nil` (for `reject`)
fn block_body_is_nil_filter(method: &str, block_body: &str) -> bool {
    match method {
        "select" | "filter" => {
            // `.select { |x| !x.nil? }` or `.select { |x| x != nil }`
            block_contains_not_nil(block_body)
        }
        "reject" => {
            // `.reject { |x| x.nil? }` or `.reject { |x| x == nil }`
            block_contains_nil(block_body)
        }
        _ => false,
    }
}

/// Returns true if the block body contains `!<ident>.nil?` or `<ident> != nil`.
fn block_contains_not_nil(body: &str) -> bool {
    // Pattern: `!something.nil?`
    if body.contains(".nil?") {
        // find `!` before `.nil?`
        if let Some(nil_pos) = body.find(".nil?") {
            // Look backwards from nil_pos for the method receiver
            let before = &body[..nil_pos];
            // The `!` should appear in the before part as negation
            if before.contains('!') {
                return true;
            }
        }
    }
    // Pattern: `something != nil`
    if body.contains("!= nil") {
        return true;
    }
    false
}

/// Returns true if the block body contains `<ident>.nil?` (not negated) or `<ident> == nil`.
fn block_contains_nil(body: &str) -> bool {
    // Pattern: `x.nil?` without preceding `!`
    if body.contains(".nil?") {
        if let Some(nil_pos) = body.find(".nil?") {
            let before = &body[..nil_pos];
            // No `!` directly before the receiver
            if !before.contains('!') {
                return true;
            }
        }
    }
    // Pattern: `something == nil`
    if body.contains("== nil") {
        return true;
    }
    false
}

/// Extract the inline block content between the first `{` and `}` on a line.
/// Returns `None` if the braces are not balanced or not found.
fn extract_inline_block(after_method: &str) -> Option<&str> {
    let open = after_method.find('{')?;
    let rest = &after_method[open + 1..];
    // Find the matching closing brace (simple, no nesting needed for this pattern)
    let close = rest.find('}')?;
    Some(&rest[..close])
}

impl Rule for CollectionCompact {
    fn name(&self) -> &'static str {
        "Style/CollectionCompact"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let line_start = ctx.line_start_offsets[i] as usize;

            // Skip full-line comments.
            if trimmed.starts_with('#') {
                continue;
            }

            // Look for `.select`, `.reject`, or `.filter` with a block `{ ... }`
            for method in &["select", "reject", "filter"] {
                let needle = format!(".{} {{", method);
                if let Some(method_pos) = line.find(&needle) {
                    let after_method = &line[method_pos + ".".len() + method.len()..];
                    if let Some(block_body) = extract_inline_block(after_method) {
                        if block_body_is_nil_filter(method, block_body) {
                            let pos = (line_start + indent) as u32;
                            diags.push(Diagnostic {
                                rule: self.name(),
                                message: "Use compact instead of rejecting nil values manually."
                                    .into(),
                                range: TextRange::new(pos, pos + line.trim_end().len() as u32),
                                severity: Severity::Warning,
                            });
                            break; // one diagnostic per line
                        }
                    }
                }
            }
        }

        diags
    }
}
