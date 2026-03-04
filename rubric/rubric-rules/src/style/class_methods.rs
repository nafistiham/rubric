use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClassMethods;

/// Scope kind pushed onto the context stack.
#[derive(Clone)]
enum Scope {
    /// A named `class Foo` or `module Foo` scope, storing the simple name.
    Named(String),
    /// Any other block/def/do that contributes an `end` but is not class/module.
    Other,
}

impl Rule for ClassMethods {
    fn name(&self) -> &'static str {
        "Style/ClassMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Stack tracking scopes that each need a matching `end`.
        let mut scope_stack: Vec<Scope> = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect scope-opening keywords.
            if let Some(name) = extract_class_or_module_name(trimmed) {
                scope_stack.push(Scope::Named(name));
            } else if is_other_block_opener(trimmed) {
                scope_stack.push(Scope::Other);
            }

            // Detect `def ReceiverName.method_name` lines.
            // We look for `def ` followed by an uppercase-starting identifier, a dot, then
            // a method name.  When the receiver name exactly matches the immediately enclosing
            // named scope (class or module), we flag it.
            if let Some(receiver) = extract_named_def_receiver(trimmed) {
                let innermost_name = scope_stack.iter().rev().find_map(|s| match s {
                    Scope::Named(n) => Some(n.as_str()),
                    Scope::Other => None,
                });
                if let Some(enc_name) = innermost_name {
                    if receiver == enc_name {
                        let indent = line.len() - trimmed.len();
                        let line_start = ctx.line_start_offsets[i] as usize;
                        let pos = (line_start + indent) as u32;
                        diags.push(Diagnostic {
                            rule: self.name(),
                            message: format!(
                                "Use `self.{}` instead of `{}.{}`.",
                                extract_method_name_after_dot(trimmed),
                                receiver,
                                extract_method_name_after_dot(trimmed),
                            ),
                            range: TextRange::new(pos, pos + trimmed.len() as u32),
                            severity: Severity::Warning,
                        });
                    }
                }

                // `def ReceiverName.method` opens a method body — push Other.
                scope_stack.push(Scope::Other);
            } else if is_def_self_opener(trimmed) || is_plain_def_opener(trimmed) {
                // `def self.method` or `def method` — opens a body.
                scope_stack.push(Scope::Other);
            }

            // Detect `end` — pops the innermost scope.
            if is_end_line(trimmed) {
                scope_stack.pop();
            }
        }

        diags
    }
}

/// If the trimmed line is a `class` or `module` opener, return the simple name
/// (the identifier immediately after the keyword, stripping any `::Namespace` prefix
/// and any inheritance `< SuperClass`).
///
/// Examples:
///   "class Foo"            => Some("Foo")
///   "class Foo < Bar"      => Some("Foo")
///   "class Foo::Bar"       => Some("Foo::Bar")  — keep full compound name
///   "module Foo"           => Some("Foo")
///   "module Foo::Bar"      => Some("Foo::Bar")
///   "class << self"        => None  (singleton class — no named scope)
fn extract_class_or_module_name(trimmed: &str) -> Option<String> {
    let rest = if trimmed.starts_with("class ") {
        &trimmed["class ".len()..]
    } else if trimmed.starts_with("module ") {
        &trimmed["module ".len()..]
    } else {
        return None;
    };

    // `class << self` is a singleton-class opener, not a named scope.
    if rest.starts_with('<') {
        return None;
    }

    // The name ends at whitespace (e.g. before `< SuperClass`) or at `#` (comment) or end.
    let name_end = rest
        .find(|c: char| c.is_whitespace() || c == '#' || c == ';')
        .unwrap_or(rest.len());
    let name = &rest[..name_end];

    if name.is_empty() {
        return None;
    }

    Some(name.to_owned())
}

/// Extract the receiver name from a `def ReceiverName.method_name(...)` line.
/// Returns `Some(receiver)` only when the receiver starts with an uppercase letter
/// (i.e. it looks like a constant/class name, not `self`).
///
/// Returns `None` for `def self.method`, `def method`, etc.
fn extract_named_def_receiver(trimmed: &str) -> Option<String> {
    // Must start with `def `.
    let rest = trimmed.strip_prefix("def ")?;

    // Skip `self.` — that is idiomatic Ruby, not flagged.
    if rest.starts_with("self.") {
        return None;
    }

    // Find the dot — must have one for a named receiver def.
    let dot_pos = rest.find('.')?;
    let receiver = &rest[..dot_pos];

    // Receiver must be a constant-like name: starts with uppercase letter and
    // contains only word chars, colons (for `Foo::Bar`), etc.
    if receiver.is_empty() {
        return None;
    }
    let first_char = receiver.chars().next()?;
    if !first_char.is_uppercase() {
        return None;
    }

    Some(receiver.to_owned())
}

/// Extract the method name that appears after the dot in `def ReceiverName.method_name(...)`.
/// Used purely for the diagnostic message.
fn extract_method_name_after_dot(trimmed: &str) -> &str {
    // Strip `def `.
    let rest = trimmed.strip_prefix("def ").unwrap_or(trimmed);
    if let Some(dot_pos) = rest.find('.') {
        let after_dot = &rest[dot_pos + 1..];
        // Method name ends at `(`, ` `, `;`, end.
        let end = after_dot
            .find(|c: char| c == '(' || c.is_whitespace() || c == ';')
            .unwrap_or(after_dot.len());
        &after_dot[..end]
    } else {
        rest
    }
}

/// Returns true if the trimmed line is `def self.method_name...`.
fn is_def_self_opener(trimmed: &str) -> bool {
    trimmed.starts_with("def self.")
}

/// Returns true if the trimmed line is a plain `def method` (not `def self.` or `def Name.`).
fn is_plain_def_opener(trimmed: &str) -> bool {
    if !trimmed.starts_with("def ") && trimmed != "def" {
        return false;
    }
    // Exclude `def self.` and `def Name.` — those are handled separately.
    if trimmed.starts_with("def self.") {
        return false;
    }
    if let Some(rest) = trimmed.strip_prefix("def ") {
        if rest.contains('.') {
            return false;
        }
    }
    true
}

/// Returns true if the trimmed line introduces a block that will need an `end`.
fn is_other_block_opener(trimmed: &str) -> bool {
    for kw in &["begin", "if ", "unless ", "while ", "until ", "for ", "case "] {
        if trimmed.starts_with(kw) {
            return true;
        }
    }
    // Inline `do` at end of line (e.g. `foo.each do`)
    if trimmed.ends_with(" do") || trimmed.contains(" do |") || trimmed.contains(" do\n") {
        return true;
    }
    // Trailing `begin` at end of line (e.g. `@@x ||= begin`).
    if trimmed.ends_with(" begin") {
        return true;
    }
    // Assignment-expression openers: `var = if ...`, `var = case ...`, `var = unless ...`.
    for kw in &["= if ", "= unless ", "= case "] {
        if trimmed.contains(kw) {
            return true;
        }
    }
    false
}

/// Returns true when this line closes a scope (bare `end` line).
fn is_end_line(trimmed: &str) -> bool {
    trimmed == "end"
        || trimmed.starts_with("end ")
        || trimmed.starts_with("end\t")
        || trimmed.starts_with("end#")
}
