use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct ClassMethods;

/// Scope kind pushed onto the context stack.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Class,
    Module,
    /// Any other block/def/do that contributes an `end` but is not class/module.
    Other,
}

impl Rule for ClassMethods {
    fn name(&self) -> &'static str {
        "Style/ClassMethods"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // Stack tracking the innermost class/module/other scope.
        let mut scope_stack: Vec<Scope> = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Skip comment lines.
            if trimmed.starts_with('#') {
                continue;
            }

            // Detect scope-opening keywords. We check the start of the
            // trimmed line so we don't fire on variable names like `classes`.
            //
            // We intentionally ignore single-line forms like `class Foo; end`
            // and `module Foo; end` (treating them as openers that expect a
            // later `end`) — these are rare in real code and skipping them
            // avoids the complexity of on-the-fly `end`-counting within a line.
            if is_class_opener(trimmed) {
                scope_stack.push(Scope::Class);
            } else if is_module_opener(trimmed) {
                scope_stack.push(Scope::Module);
            } else if is_other_block_opener(trimmed) {
                scope_stack.push(Scope::Other);
            }

            // Detect def self.method lines.
            if trimmed.starts_with("def self.") {
                // Only flag when the immediately enclosing scope is a module.
                let innermost = scope_stack.last().copied();
                if innermost == Some(Scope::Module) {
                    let indent = line.len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "Use `module_function` or `extend self` instead of `def self.method_name` inside a module.".into(),
                        range: TextRange::new(pos, pos + trimmed.len() as u32),
                        severity: Severity::Warning,
                    });
                }

                // `def self.method` opens a method body — push Other so the
                // matching `end` will pop it correctly.
                scope_stack.push(Scope::Other);
            } else if is_def_opener(trimmed) {
                // Plain `def method` — also opens a body.
                scope_stack.push(Scope::Other);
            }

            // Detect `end` — pops the innermost scope.
            // We match lines that are exactly `end` (possibly followed by
            // comment) and lines like `end; foo` which are unusual but valid.
            if is_end_line(trimmed) {
                scope_stack.pop();
            }
        }

        diags
    }
}

/// Returns true if the trimmed line opens a class body.
/// Matches: `class Foo`, `class Foo < Bar`, `class << self`
fn is_class_opener(trimmed: &str) -> bool {
    if !trimmed.starts_with("class") {
        return false;
    }
    let after = &trimmed["class".len()..];
    // Must be followed by whitespace or `<` (for `class << self`)
    after.starts_with(|c: char| c.is_whitespace() || c == '<')
}

/// Returns true if the trimmed line opens a module body.
/// Matches: `module Foo`, `module Foo::Bar`
fn is_module_opener(trimmed: &str) -> bool {
    if !trimmed.starts_with("module") {
        return false;
    }
    let after = &trimmed["module".len()..];
    after.starts_with(|c: char| c.is_whitespace())
}

/// Returns true if the trimmed line is a plain `def` (not `def self.`).
fn is_def_opener(trimmed: &str) -> bool {
    if !trimmed.starts_with("def ") && trimmed != "def" {
        return false;
    }
    // Exclude `def self.` — that case is handled separately above.
    !trimmed.starts_with("def self.")
}

/// Returns true if the trimmed line introduces a block that will need an `end`.
/// Covers: `do`, `if/unless/while/until/for/begin` on their own line,
/// `do |...| ... end` single-line is intentionally excluded.
fn is_other_block_opener(trimmed: &str) -> bool {
    // We intentionally keep this minimal so we don't accidentally mis-count.
    // The critical ones are keywords that always have a matching `end`:
    for kw in &["do", "begin", "if ", "unless ", "while ", "until ", "for ", "case "] {
        if trimmed.starts_with(kw) {
            return true;
        }
    }
    // Inline `do` at end of line (e.g. `foo.each do`)
    if trimmed.ends_with(" do") || trimmed.contains(" do |") || trimmed.contains(" do\n") {
        return true;
    }
    // Trailing `begin` at end of line (e.g. `@@x ||= begin`).
    // This covers Ruby's inline begin..end expression form where `begin` appears
    // as the last token on a line rather than at the start.  Without this check
    // the matching `end` is consumed by the wrong scope entry and the stack
    // falls out of sync, causing subsequent `def self.method` inside the same
    // class to be mis-flagged as if they were inside a module.
    if trimmed.ends_with(" begin") {
        return true;
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
