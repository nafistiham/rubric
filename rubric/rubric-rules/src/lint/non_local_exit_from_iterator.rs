use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct NonLocalExitFromIterator;

/// Returns true if a trimmed line opens a `do`-block that is a lambda or
/// define_method — inside these, `return` is a local exit, not a non-local exit.
fn is_lambda_or_define_method_do(t: &str) -> bool {
    // `lambda do ...` or `CONST = lambda do ...`
    if t.contains("lambda do") {
        return true;
    }
    // `define_method(...) do` or `define_method :name do`
    if t.starts_with("define_method") && is_do_block_opener(t) {
        return true;
    }
    // `let!(:name) do` — RSpec memoisation helper, return is a value expression
    // rubocop allows this.
    if (t.starts_with("let(") || t.starts_with("let!(") || t.starts_with("let! (")
        || t.starts_with("let ("))
        && is_do_block_opener(t)
    {
        return true;
    }
    false
}

/// Returns true if the line contains a `do` block opener (` do`, ` do |`, ` do|`)
/// with proper word-boundary handling — does NOT match `doc`, `domain`, etc.
fn is_do_block_opener(t: &str) -> bool {
    // Ends with ` do` (word boundary guaranteed by end-of-string)
    if t.ends_with(" do") {
        return true;
    }
    // Contains ` do |` or ` do|` — `do` followed by `|` or space+`|`
    if t.contains(" do |") || t.contains(" do|") {
        return true;
    }
    false
}

/// Stack frame kind for tracking block/def nesting.
#[derive(Clone, Copy, PartialEq)]
enum Frame {
    /// A `def`/`class`/`module` — `return` inside is always local.
    Def,
    /// A regular iterator/block `do...end` — `return` is a non-local exit.
    Block,
    /// A lambda or define_method `do...end` — `return` is a local exit.
    LambdaOrMethod,
    /// An `if`/`unless`/`while`/`until`/`for`/`case`/`begin` — tracks `end` consumption
    /// but does not affect whether `return` is flagged.
    Other,
}

impl Rule for NonLocalExitFromIterator {
    fn name(&self) -> &'static str {
        "Lint/NonLocalExitFromIterator"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let lines = &ctx.lines;
        let n = lines.len();

        // Stack of frames tracking the nesting of Ruby constructs.
        // We need a proper stack (not just counters) so that every `end` token
        // pops the correct frame — including `if/begin/etc` that don't affect
        // def_depth or block_depth directly.
        let mut stack: Vec<Frame> = Vec::new();

        for i in 0..n {
            let trimmed = lines[i].trim_start();
            let t = trimmed.trim();

            if t.starts_with('#') {
                continue;
            }

            // Track def/class/module openings — a return inside a nested def is not non-local.
            let is_def_opener = (t.starts_with("def ") || t == "def") && !t.starts_with("defined?")
                || t.starts_with("class ") || t == "class"
                || t.starts_with("module ") || t == "module";

            // Track do-block openings — check BEFORE def because we need `is_do_block_opener`.
            // Only flag as iterator if we are NOT currently inside a def frame.
            let is_do_opener = !is_def_opener && is_do_block_opener(t);

            if is_def_opener {
                stack.push(Frame::Def);
            } else if is_do_opener {
                if is_lambda_or_define_method_do(t) {
                    stack.push(Frame::LambdaOrMethod);
                } else {
                    // Regular iterator block — but only flag `return` if we're not
                    // already inside a Def frame on the stack (which would shadow).
                    stack.push(Frame::Block);
                }
            } else if t.starts_with("if ") || t == "if"
                || t.starts_with("unless ") || t == "unless"
                || t.starts_with("while ") || t == "while"
                || t.starts_with("until ") || t == "until"
                || t.starts_with("for ") || t == "begin"
                || t.starts_with("begin ") || t.starts_with("case ")
                || t == "case"
            {
                stack.push(Frame::Other);
            }

            // Consume `end` — only bare `end` (not `end.method`, `end,` etc.)
            if t == "end" {
                stack.pop();
            }

            // Detect `return` — flag only if the innermost Def/Block/LambdaOrMethod
            // frame is a Block (iterator), with no intervening Def or LambdaOrMethod.
            if t.starts_with("return ") || t == "return" {
                // Walk the stack from top to find the nearest semantically-relevant frame.
                let in_iterator = stack.iter().rev().any(|f| matches!(f, Frame::Block))
                    && !stack
                        .iter()
                        .rev()
                        .any(|f| matches!(f, Frame::Def | Frame::LambdaOrMethod));

                if in_iterator {
                    let indent = lines[i].len() - trimmed.len();
                    let line_start = ctx.line_start_offsets[i] as usize;
                    let pos = (line_start + indent) as u32;
                    diags.push(Diagnostic {
                        rule: self.name(),
                        message: "`return` inside iterator block causes non-local exit.".into(),
                        range: TextRange::new(pos, pos + t.len() as u32),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        diags
    }
}
