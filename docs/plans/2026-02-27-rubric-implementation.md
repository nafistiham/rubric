# Rubric Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build Rubric — a Ruby linter and formatter in Rust that replaces Rubocop, shipping ~150 cops as a gem with precompiled binaries.

**Architecture:** Unified engine (Approach A / Ruff Blueprint) — each cop implements a `Rule` trait with an optional `fix()` method. Formatter mode is `rubric fmt` = running all safe formatting fixes. Files processed in parallel via Rayon. AST provided by `ruby-prism` (official Ruby parser, Rust bindings).

**Tech Stack:** Rust (2021 edition), `ruby-prism = "1.7.0"`, `rayon`, `clap 4`, `serde + toml`, `anyhow`, `walkdir`. Gem distribution via `rb_sys` + `rake-compiler`.

---

## Milestone 1: Foundation

Workspace + core types + Rule trait + first cop (TrailingWhitespace) + test harness + `rubric check` CLI.

---

### Task 1: Initialize Cargo Workspace

**Files:**
- Create: `rubric/Cargo.toml`
- Create: `rubric/rubric-core/Cargo.toml`
- Create: `rubric/rubric-rules/Cargo.toml`
- Create: `rubric/rubric-cli/Cargo.toml`

**Step 1: Create the project directory**

```bash
mkdir -p rubric
cd rubric
```

**Step 2: Write workspace `Cargo.toml`**

```toml
# rubric/Cargo.toml
[workspace]
members = [
    "rubric-core",
    "rubric-rules",
    "rubric-cli",
]
resolver = "2"

[workspace.dependencies]
ruby-prism = "1.7.0"
rayon      = "1.10"
anyhow     = "1"
clap       = { version = "4", features = ["derive"] }
serde      = { version = "1", features = ["derive"] }
toml       = "0.8"
walkdir    = "2"
```

**Step 3: Create each crate**

```bash
cargo new --lib rubric-core
cargo new --lib rubric-rules
cargo new --bin rubric-cli
```

**Step 4: Write `rubric-core/Cargo.toml`**

```toml
[package]
name    = "rubric-core"
version = "0.1.0"
edition = "2021"

[dependencies]
ruby-prism = { workspace = true }
```

**Step 5: Write `rubric-rules/Cargo.toml`**

```toml
[package]
name    = "rubric-rules"
version = "0.1.0"
edition = "2021"

[dependencies]
rubric-core = { path = "../rubric-core" }
ruby-prism  = { workspace = true }

[dev-dependencies]
rubric-core = { path = "../rubric-core" }
```

**Step 6: Write `rubric-cli/Cargo.toml`**

```toml
[package]
name    = "rubric-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
rubric-core  = { path = "../rubric-core" }
rubric-rules = { path = "../rubric-rules" }
ruby-prism   = { workspace = true }
clap         = { workspace = true }
anyhow       = { workspace = true }
walkdir      = { workspace = true }
```

**Step 7: Verify workspace builds**

```bash
cargo build
```

Expected: Compiles all three crates with zero errors.

**Step 8: Commit**

```bash
git init
echo "target/" > .gitignore
git add .
git commit -m "feat: initialize rubric cargo workspace with three crates"
```

---

### Task 2: Define Core Types

**Files:**
- Create: `rubric-core/src/types.rs`
- Modify: `rubric-core/src/lib.rs`

**Step 1: Write `rubric-core/src/types.rs`**

```rust
/// A byte-offset range within a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

impl TextRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A lint violation reported by a rule.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rule: &'static str,  // e.g. "Layout/TrailingWhitespace"
    pub message: String,
    pub range: TextRange,
    pub severity: Severity,
}

/// A single text substitution that resolves a violation.
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

/// Whether applying this fix could change program behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixSafety {
    Safe,
    Unsafe,
}

/// A complete auto-fix for a diagnostic (may contain multiple edits).
#[derive(Debug, Clone)]
pub struct Fix {
    pub edits: Vec<TextEdit>,
    pub safety: FixSafety,
}
```

**Step 2: Update `rubric-core/src/lib.rs`**

```rust
pub mod types;

pub use types::{Diagnostic, Fix, FixSafety, Severity, TextEdit, TextRange};
```

**Step 3: Compile**

```bash
cargo build -p rubric-core
```

Expected: Clean build, no warnings.

**Step 4: Commit**

```bash
git add rubric-core/src/
git commit -m "feat: define core types (Diagnostic, Fix, TextRange, Severity)"
```

---

### Task 3: Define `LintContext`

**Files:**
- Create: `rubric-core/src/context.rs`
- Modify: `rubric-core/src/lib.rs`

**Step 1: Write the failing test first**

Add to `rubric-core/src/context.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_context_splits_lines() {
        let source = "def foo\n  bar\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.lines.len(), 3);
        assert_eq!(ctx.lines[0], "def foo");
        assert_eq!(ctx.lines[1], "  bar");
        assert_eq!(ctx.lines[2], "end");
    }

    #[test]
    fn test_context_line_start_offsets() {
        let source = "ab\ncd\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        // "ab" starts at 0, "cd" starts at 3 (after "ab\n")
        assert_eq!(ctx.line_start_offsets[0], 0);
        assert_eq!(ctx.line_start_offsets[1], 3);
    }
}
```

**Step 2: Run test — confirm it fails**

```bash
cargo test -p rubric-core
```

Expected: FAIL — `LintContext` not defined.

**Step 3: Implement `LintContext`**

Write `rubric-core/src/context.rs`:

```rust
use std::path::Path;

/// Per-file context passed to every rule during a lint run.
pub struct LintContext<'src> {
    pub path: &'src Path,
    pub source: &'src str,
    /// Lines of source (without newlines).
    pub lines: Vec<&'src str>,
    /// Byte offset of the start of each line (index = line number, 0-based).
    pub line_start_offsets: Vec<u32>,
}

impl<'src> LintContext<'src> {
    pub fn new(path: &'src Path, source: &'src str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut offset: u32 = 0;
        for line in &lines {
            offsets.push(offset);
            offset += line.len() as u32 + 1; // +1 for '\n'
        }
        Self {
            path,
            source,
            lines,
            line_start_offsets: offsets,
        }
    }

    /// Convert a byte offset to (line, column), both 1-based.
    pub fn offset_to_line_col(&self, offset: u32) -> (usize, usize) {
        let line_idx = self
            .line_start_offsets
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let col = (offset - self.line_start_offsets[line_idx]) as usize + 1;
        (line_idx + 1, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_context_splits_lines() {
        let source = "def foo\n  bar\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.lines.len(), 3);
        assert_eq!(ctx.lines[0], "def foo");
        assert_eq!(ctx.lines[1], "  bar");
        assert_eq!(ctx.lines[2], "end");
    }

    #[test]
    fn test_context_line_start_offsets() {
        let source = "ab\ncd\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.line_start_offsets[0], 0);
        assert_eq!(ctx.line_start_offsets[1], 3);
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "ab\ncd\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.offset_to_line_col(0), (1, 1)); // 'a'
        assert_eq!(ctx.offset_to_line_col(1), (1, 2)); // 'b'
        assert_eq!(ctx.offset_to_line_col(3), (2, 1)); // 'c'
    }
}
```

**Step 4: Export from `lib.rs`**

```rust
pub mod context;
pub mod types;

pub use context::LintContext;
pub use types::{Diagnostic, Fix, FixSafety, Severity, TextEdit, TextRange};
```

**Step 5: Run tests — confirm pass**

```bash
cargo test -p rubric-core
```

Expected: 3 tests pass.

**Step 6: Commit**

```bash
git add rubric-core/src/
git commit -m "feat: add LintContext with line offsets and position helpers"
```

---

### Task 4: Define `Rule` Trait

**Files:**
- Create: `rubric-core/src/rule.rs`
- Modify: `rubric-core/src/lib.rs`

**Step 1: Write `rubric-core/src/rule.rs`**

```rust
use crate::{context::LintContext, types::{Diagnostic, Fix}};

/// Every Rubric cop implements this trait.
///
/// - Source-level rules (line length, trailing whitespace): implement `check_source`.
/// - AST-level rules (string literals, method style): implement `check_node` (M2+).
pub trait Rule: Send + Sync {
    /// Rubocop-style identifier, e.g. "Layout/TrailingWhitespace".
    fn name(&self) -> &'static str;

    /// Called once per file with the full source.
    /// Override for line-level or whole-file checks.
    fn check_source(&self, _ctx: &LintContext) -> Vec<Diagnostic> {
        vec![]
    }

    /// Produce a fix for the given diagnostic.
    /// Returns `None` if this rule has no auto-fix.
    fn fix(&self, _diag: &Diagnostic) -> Option<Fix> {
        None
    }
}
```

**Step 2: Export from `lib.rs`**

```rust
pub mod context;
pub mod rule;
pub mod types;

pub use context::LintContext;
pub use rule::Rule;
pub use types::{Diagnostic, Fix, FixSafety, Severity, TextEdit, TextRange};
```

**Step 3: Build**

```bash
cargo build -p rubric-core
```

Expected: Clean.

**Step 4: Commit**

```bash
git add rubric-core/src/rule.rs rubric-core/src/lib.rs
git commit -m "feat: define Rule trait for source-level and AST-level cops"
```

---

### Task 5: Implement `Layout/TrailingWhitespace`

**Files:**
- Create: `rubric-rules/src/layout/mod.rs`
- Create: `rubric-rules/src/layout/trailing_whitespace.rs`
- Modify: `rubric-rules/src/lib.rs`

**Step 1: Create fixture files**

```bash
mkdir -p rubric-rules/tests/fixtures/layout/trailing_whitespace
```

Write `rubric-rules/tests/fixtures/layout/trailing_whitespace/offending.rb`:
```ruby
def hello
  puts "world"
end
```
> Note: lines 1 and 2 have trailing spaces (3 and 2 respectively). Ensure they are preserved when saving.

Write `rubric-rules/tests/fixtures/layout/trailing_whitespace/corrected.rb`:
```ruby
def hello
  puts "world"
end
```

**Step 2: Write the failing test**

Create `rubric-rules/tests/trailing_whitespace_test.rs`:

```rust
use rubric_core::{LintContext, Diagnostic};
use rubric_rules::layout::trailing_whitespace::TrailingWhitespace;
use rubric_core::Rule;
use std::path::Path;

const OFFENDING: &str =
    include_str!("fixtures/layout/trailing_whitespace/offending.rb");
const CORRECTED: &str =
    include_str!("fixtures/layout/trailing_whitespace/corrected.rb");

#[test]
fn detects_trailing_whitespace() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diagnostics = TrailingWhitespace.check_source(&ctx);

    assert_eq!(diagnostics.len(), 2, "expected 2 violations (lines 1 and 2)");
    assert!(diagnostics.iter().all(|d| d.rule == "Layout/TrailingWhitespace"));
}

#[test]
fn fix_removes_trailing_whitespace() {
    use rubric_core::{TextEdit};

    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diagnostics = TrailingWhitespace.check_source(&ctx);

    // Apply all fixes to source
    let mut result = OFFENDING.to_string();
    // Apply edits in reverse order to preserve offsets
    let mut edits: Vec<_> = diagnostics
        .iter()
        .filter_map(|d| TrailingWhitespace.fix(d))
        .flat_map(|f| f.edits)
        .collect();
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    for edit in edits {
        let start = edit.range.start as usize;
        let end   = edit.range.end as usize;
        result.replace_range(start..end, &edit.replacement);
    }

    assert_eq!(result, CORRECTED);
}
```

**Step 3: Run — confirm fails**

```bash
cargo test -p rubric-rules
```

Expected: FAIL — `rubric_rules::layout::trailing_whitespace::TrailingWhitespace` not found.

**Step 4: Implement the cop**

Create `rubric-rules/src/layout/mod.rs`:
```rust
pub mod trailing_whitespace;
```

Create `rubric-rules/src/layout/trailing_whitespace.rs`:

```rust
use rubric_core::{Diagnostic, Fix, FixSafety, LintContext, Rule, Severity, TextEdit, TextRange};

pub struct TrailingWhitespace;

impl Rule for TrailingWhitespace {
    fn name(&self) -> &'static str {
        "Layout/TrailingWhitespace"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed_len = line.trim_end().len();
            let trailing = line.len() - trimmed_len;
            if trailing == 0 {
                continue;
            }
            let line_start = ctx.line_start_offsets[i];
            let start = line_start + trimmed_len as u32;
            let end   = line_start + line.len() as u32;
            diagnostics.push(Diagnostic {
                rule: self.name(),
                message: "Trailing whitespace detected.".into(),
                range: TextRange::new(start, end),
                severity: Severity::Warning,
            });
        }

        diagnostics
    }

    fn fix(&self, diag: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            edits: vec![TextEdit {
                range: diag.range,
                replacement: String::new(),
            }],
            safety: FixSafety::Safe,
        })
    }
}
```

Update `rubric-rules/src/lib.rs`:

```rust
pub mod layout;

pub use layout::trailing_whitespace::TrailingWhitespace;
```

**Step 5: Run — confirm passes**

```bash
cargo test -p rubric-rules
```

Expected: 2 tests pass.

**Step 6: Commit**

```bash
git add rubric-rules/
git commit -m "feat: implement Layout/TrailingWhitespace with auto-fix"
```

---

### Task 6: Build `rubric check` CLI

**Files:**
- Create: `rubric-cli/src/main.rs`
- Create: `rubric-cli/src/runner.rs`

**Step 1: Write `rubric-cli/src/runner.rs`**

```rust
use std::path::Path;
use anyhow::Result;
use walkdir::WalkDir;
use rubric_core::{LintContext, Rule};

pub fn collect_ruby_files(path: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("rb")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn run_rules_on_file(
    path: &Path,
    rules: &[Box<dyn Rule>],
) -> Result<Vec<rubric_core::Diagnostic>> {
    let source = std::fs::read_to_string(path)?;
    let ctx = LintContext::new(path, &source);
    let mut diagnostics = Vec::new();
    for rule in rules {
        diagnostics.extend(rule.check_source(&ctx));
    }
    Ok(diagnostics)
}
```

**Step 2: Write `rubric-cli/src/main.rs`**

```rust
mod runner;

use clap::{Parser, Subcommand};
use anyhow::Result;
use rubric_core::Rule;
use rubric_rules::TrailingWhitespace;

#[derive(Parser)]
#[command(name = "rubric", version, about = "A fast Ruby linter and formatter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint Ruby files and report violations
    Check {
        /// Path to lint (file or directory)
        #[arg(default_value = ".")]
        path: std::path::PathBuf,

        /// Apply safe auto-fixes
        #[arg(long)]
        fix: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path, fix } => {
            let rules: Vec<Box<dyn Rule>> = vec![
                Box::new(TrailingWhitespace),
            ];

            let files = runner::collect_ruby_files(&path);

            if files.is_empty() {
                println!("No Ruby files found.");
                return Ok(());
            }

            let mut total_violations = 0;

            for file in &files {
                let diagnostics = runner::run_rules_on_file(file, &rules)?;
                for diag in &diagnostics {
                    let (line, col) = {
                        let source = std::fs::read_to_string(file)?;
                        let ctx = rubric_core::LintContext::new(file, &source);
                        ctx.offset_to_line_col(diag.range.start)
                    };
                    println!(
                        "{}:{}:{}: [{}] {} ({})",
                        file.display(),
                        line,
                        col,
                        format!("{:?}", diag.severity).to_uppercase(),
                        diag.message,
                        diag.rule
                    );
                }
                total_violations += diagnostics.len();
            }

            if total_violations > 0 {
                eprintln!("\n{} violation(s) found.", total_violations);
                std::process::exit(1);
            } else {
                println!("No violations found.");
            }
        }
    }

    Ok(())
}
```

**Step 3: Build and smoke test**

```bash
cargo build -p rubric-cli

# Create a test file with trailing whitespace
echo -e "def hello   \n  puts 'world'  \nend" > /tmp/test.rb

# Run rubric
./target/debug/rubric-cli check /tmp/test.rb
```

Expected output:
```
/tmp/test.rb:1:9: [WARNING] Trailing whitespace detected. (Layout/TrailingWhitespace)
/tmp/test.rb:2:15: [WARNING] Trailing whitespace detected. (Layout/TrailingWhitespace)

2 violation(s) found.
```

**Step 4: Commit**

```bash
git add rubric-cli/src/
git commit -m "feat: add rubric check CLI command with file walker"
```

---

## Milestone 2: Parallel Processing + `rubric.toml` Config + 10 Cops

### Task 7: Add Rayon Parallel File Processing

**Files:**
- Modify: `rubric-cli/src/runner.rs`
- Modify: `rubric-cli/Cargo.toml` (add `rayon`)

Add `rayon = { workspace = true }` to `rubric-cli/Cargo.toml` and `rubric-rules/Cargo.toml`.

Update `runner.rs` to use `par_iter`:

```rust
use rayon::prelude::*;

pub fn run_all_files(
    files: &[std::path::PathBuf],
    rules: &[Box<dyn Rule + Send + Sync>],
) -> Vec<(std::path::PathBuf, Vec<rubric_core::Diagnostic>)> {
    files
        .par_iter()
        .filter_map(|path| {
            run_rules_on_file(path, rules)
                .ok()
                .map(|d| (path.clone(), d))
        })
        .collect()
}
```

Test: lint the `rubric` project itself (should be fast with no violations), then lint a large Ruby file 1000x to verify parallel speedup.

Commit: `perf: add rayon parallel file processing`

---

### Task 8: `rubric.toml` Config Parsing

**Files:**
- Create: `rubric-cli/src/config.rs`

Define a `Config` struct with serde. Fields:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub linter: LinterConfig,
    #[serde(default)]
    pub formatter: FormatterConfig,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LinterConfig {
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct FormatterConfig {
    pub enabled: Option<bool>,
}
```

Load `rubric.toml` from the current directory. Fall back to defaults if not found.

TDD: write a test that creates a temp dir with a `rubric.toml`, asserts `Config::load()` parses it correctly.

Commit: `feat: add rubric.toml config loading`

---

### Task 9: AST Walker for Node-Based Rules

**Files:**
- Create: `rubric-core/src/walker.rs`

Implement an AST walker using `ruby_prism`'s `Visit` trait. The walker holds enabled rules and dispatches each visited node to rules that registered interest via `node_kinds()`.

Extend the `Rule` trait:

```rust
pub fn check_node(&self, _ctx: &LintContext, _node: &ruby_prism::Node) -> Vec<Diagnostic> {
    vec![]
}
pub fn node_kinds(&self) -> &[&'static str] {
    &[]   // empty = not an AST rule; return node class names like &["StringNode"]
}
```

The walker implements `ruby_prism::Visit` and in each `visit_*` method dispatches to registered rules.

TDD: write a test that parses `"hello"` and asserts the walker visits a `StringNode`.

Commit: `feat: add AST walker with ruby-prism Visit trait integration`

---

### Task 10: 9 More Cops (Total: 10)

Implement one cop at a time, each with fixture files and tests. Order by simplicity:

1. `Layout/TrailingNewlines` — source-level, detect missing final newline
2. `Layout/IndentationWidth` — source-level, detect non-2-space indentation
3. `Layout/LineLength` — source-level, lines > 120 chars
4. `Layout/EmptyLines` — source-level, multiple consecutive blank lines
5. `Layout/SpaceAfterComma` — source-level, `foo,bar` vs `foo, bar`
6. `Style/StringLiterals` — AST-level via Walker, double quotes when single suffices
7. `Style/FrozenStringLiteralComment` — source-level, missing `# frozen_string_literal: true`
8. `Lint/UnusedVariable` — AST-level, local vars assigned but never read
9. `Style/TrailingCommaInArguments` — AST-level, trailing comma in method args

**For each cop:**
1. Create `rubric-rules/tests/fixtures/<dept>/<cop_name>/offending.rb`
2. Create `rubric-rules/tests/fixtures/<dept>/<cop_name>/corrected.rb` (if fixable)
3. Write failing test in `rubric-rules/tests/<cop_name>_test.rs`
4. Run test — confirm FAIL
5. Implement cop in `rubric-rules/src/<dept>/<cop_name>.rs`
6. Run test — confirm PASS
7. Register cop in `rubric-rules/src/lib.rs`
8. Commit: `feat: implement <Dept>/<CopName>`

---

## Milestone 3: Auto-Fix Engine + `rubric fmt` + 30 Cops

### Task 11: Apply-Fixes Engine

**Files:**
- Create: `rubric-core/src/apply_fixes.rs`

```rust
/// Apply a set of non-overlapping fixes to source, returning the corrected string.
/// Panics if edits overlap (bug in rule implementation).
pub fn apply_fixes(source: &str, fixes: &[Fix]) -> String {
    let mut edits: Vec<TextEdit> = fixes.iter()
        .flat_map(|f| f.edits.iter().cloned())
        .collect();
    // Sort descending by start offset so earlier edits don't shift later ones
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    let mut result = source.to_string();
    for edit in edits {
        result.replace_range(edit.range.start as usize..edit.range.end as usize, &edit.replacement);
    }
    result
}
```

TDD: test with multiple overlapping-region edits that should not occur, and non-overlapping edits that should merge correctly.

Commit: `feat: add apply_fixes engine for auto-fix mode`

---

### Task 12: `--fix` Flag + `rubric fmt` Command

Add `--fix` to `rubric check` (apply safe fixes, write back to disk).

Add `rubric fmt [path]` subcommand = `rubric check --fix` scoped to Layout/* + Style/* rules only.

TDD: integration test — copy offending fixture to a temp file, run `rubric check --fix`, assert file matches corrected fixture.

Commit: `feat: add --fix flag and rubric fmt subcommand`

---

### Task 13: 20 More Cops (Total: 30)

Continue cop-by-cop TDD cycle. Focus on most commonly used Layout and Style cops (see design doc `docs/plans/2026-02-27-rubric-design.md` cop list). Each cop = fixture + failing test + implementation + passing test + commit.

---

## Milestone 4: `rubric migrate` + 75 Cops

### Task 14: `rubric migrate` Command

**Files:**
- Create: `rubric-cli/src/commands/migrate.rs`

Parse `.rubocop.yml` (using `serde_yaml` crate) and emit `rubric.toml`.

Algorithm:
1. Read `.rubocop.yml`
2. For each key that matches a known Rubric cop name: emit `[rules."Dept/CopName"] = { enabled = true/false, ...config }`
3. For unknown keys: emit as TOML comment `# UNKNOWN: Dept/CopName (not yet implemented in Rubric)`
4. Write `rubric.toml`

TDD: test with a sample `.rubocop.yml` fixture, assert output `rubric.toml` content.

Commit: `feat: add rubric migrate command (.rubocop.yml → rubric.toml)`

---

### Task 15: 45 More Cops (Total: 75)

Continue cop-by-cop TDD for remaining Style, Layout, Lint cops. After each batch of 10, run the full test suite and benchmark against Rubocop on the `rubric` codebase itself.

---

## Milestone 5: 150 Cops + Benchmark + Gem Packaging

### Task 16: 75 More Cops (Total: 150)

Complete all cops from the design doc cop list. Prioritize Lint department (catches real bugs) over remaining Style/Layout.

---

### Task 17: Benchmark Suite

**Files:**
- Create: `benches/rubric_bench.rs`

Use `criterion` crate. Benchmark:
1. `rubric check` on the `rubric` repo itself (small)
2. `rubric check` on a vendored snapshot of a large Rails gem (e.g. `devise`)

Assert: must be at least 5x faster than equivalent `rubocop` invocation.

Commit: `bench: add criterion benchmark suite`

---

### Task 18: Gem Packaging

**Files:**
- Create: `gem/rubric.gemspec`
- Create: `gem/lib/rubric.rb`
- Create: `gem/Rakefile`
- Create: `.github/workflows/release.yml`

Use `rb_sys` + `rake-compiler` for cross-compilation. Platform matrix:
- `x86_64-linux`
- `aarch64-linux`
- `x86_64-darwin`
- `arm64-darwin`

Release workflow:
1. Tag `v0.1.0`
2. GitHub Actions cross-compiles for all 4 platforms
3. Pushes platform gems to RubyGems.org
4. Meta gem `rubric` declares platform-specific dependencies

Commit: `feat: add gem packaging with cross-compiled platform gems`

---

## Milestone 6: Public Release

### Task 19: README + Docs

- README.md: installation, usage, cop list, performance comparison with Rubocop
- `docs/contributing.md`: how to add a new cop (TDD fixture workflow)
- `docs/cops/`: auto-generated cop documentation

---

### Task 20: First Public Release

1. Run full test suite — zero failures
2. Run benchmarks — confirm performance targets
3. Tag `v0.1.0`
4. Publish gem to RubyGems.org
5. Post to Ruby community (Reddit r/ruby, Ruby Weekly)

---

## Open Questions (resolve before Task 9)

1. **ruby-prism Visit API** — Run `cargo doc --open -p ruby-prism` and confirm the exact method signatures for `Visit` trait. The docs say it exists; verify node iteration and whether `Node` is an enum or trait object.
2. **miette vs custom output** — Check if `miette` crate integrates cleanly for colored diagnostic output (like rustc errors). Alternative: use `codespan-reporting`.
3. **Cross-compilation** — Verify `cross` works for `aarch64-unknown-linux-gnu` target on macOS host before committing to that toolchain.
