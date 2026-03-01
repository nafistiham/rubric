# Contributing to Rubric

## How to Add a New Cop

Every cop follows the same TDD fixture workflow. Here is the complete step-by-step process.

### 1. Choose a name and department

Cops are named `Dept/CopName` following Rubocop conventions:
- `Layout/` — whitespace, indentation, formatting
- `Style/` — idiomatic Ruby patterns
- `Lint/` — real bugs and suspicious code

### 2. Create fixture files

```sh
mkdir -p rubric-rules/tests/fixtures/<dept>/<snake_case_name>/
```

Write `offending.rb` — Ruby code that should trigger the violation:
```ruby
# rubric-rules/tests/fixtures/layout/my_cop/offending.rb
def foo
  bar  # this triggers the violation
end
```

Write `corrected.rb` — the expected output after auto-fix (only if the cop implements `fix()`):
```ruby
# rubric-rules/tests/fixtures/layout/my_cop/corrected.rb
def foo
  bar  # fixed
end
```

### 3. Write the failing test

Create `rubric-rules/tests/my_cop_test.rs`:

```rust
use rubric_core::{LintContext, Rule};
use rubric_rules::layout::my_cop::MyCop;
use std::path::Path;

const OFFENDING: &str = include_str!("fixtures/layout/my_cop/offending.rb");

#[test]
fn detects_violation() {
    let ctx = LintContext::new(Path::new("test.rb"), OFFENDING);
    let diags = MyCop.check_source(&ctx);
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d.rule == "Layout/MyCop"));
}

#[test]
fn no_violation_on_clean() {
    let ctx = LintContext::new(Path::new("test.rb"), "# clean code here\n");
    let diags = MyCop.check_source(&ctx);
    assert!(diags.is_empty());
}
```

Run it — it should **fail** (type not found yet):
```sh
cargo test -p rubric-rules my_cop
```

### 4. Implement the cop

Create `rubric-rules/src/layout/my_cop.rs`:

```rust
use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct MyCop;

impl Rule for MyCop {
    fn name(&self) -> &'static str {
        "Layout/MyCop"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (i, line) in ctx.lines.iter().enumerate() {
            // your detection logic here
            if line.contains("violation_pattern") {
                let start = ctx.line_start_offsets[i];
                diags.push(Diagnostic {
                    rule: self.name(),
                    message: "Describe the violation.".into(),
                    range: TextRange::new(start, start + line.len() as u32),
                    severity: Severity::Warning,
                });
            }
        }
        diags
    }
}
```

Run the test — it should now **pass**:
```sh
cargo test -p rubric-rules my_cop
```

### 5. Register the cop

Add to `rubric-rules/src/layout/mod.rs`:
```rust
pub mod my_cop;
pub use my_cop::MyCop;
```

Add to `rubric-rules/src/lib.rs`:
```rust
pub use layout::{..., MyCop};
```

Add to `build_rules()` in `rubric-cli/src/main.rs`:
```rust
Box::new(MyCop),
```

Add to `KNOWN_COPS` in `rubric-cli/src/commands/migrate.rs`:
```rust
"Layout/MyCop",
```

### 6. Commit

```sh
git add rubric-rules/src/layout/my_cop.rs \
        rubric-rules/tests/my_cop_test.rs \
        rubric-rules/tests/fixtures/layout/my_cop/ \
        rubric-rules/src/layout/mod.rs \
        rubric-rules/src/lib.rs \
        rubric-cli/src/main.rs \
        rubric-cli/src/commands/migrate.rs
git commit -m "feat: implement Layout/MyCop"
```

## Important Guidelines

- **String context**: Many cops need to avoid false positives inside string literals. Copy the `in_string: Option<u8>` byte-scanner pattern from `rubric-rules/src/layout/space_after_comma.rs`.
- **Comment context**: Skip content after `#` on a line (unless tracking string context).
- **Use `trim_start()`** not `trim()` for indent calculations (avoids counting trailing whitespace).
- **No `unwrap()`** on anything that could fail on valid Ruby input.
- **One commit per cop** — atomic, reviewable history.

## Running Tests

```sh
cargo test                        # all crates
cargo test -p rubric-rules        # just cops
cargo test -p rubric-rules my_cop # just one cop
```

## Running Benchmarks

```sh
cargo bench                                    # full benchmark run
cargo bench -- lint_generated_2000_lines       # specific benchmark
```
