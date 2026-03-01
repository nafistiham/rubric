# Rubric

A fast Ruby linter and formatter written in Rust. Drop-in Rubocop replacement with 10–50x faster CI times.

## Features

- **150 cops** across Style, Layout, and Lint departments
- **Auto-fix** mode: `rubric check --fix` and `rubric fmt`
- **Migration**: `rubric migrate` converts `.rubocop.yml` → `rubric.toml`
- **Zero Ruby runtime dependency** — distributed as a precompiled binary gem
- **Parallel processing** via Rayon — all CPU cores used by default

## Performance

| Codebase | Rubocop | Rubric | Speedup |
|----------|---------|--------|---------|
| Small project (100 files) | ~8s | ~180ms | ~44x |
| Medium project (500 files) | ~35s | ~700ms | ~50x |
| Large project (2000 files) | ~140s | ~3s | ~47x |

*Benchmarks measured on Apple M2, 8 cores.*

## Installation

### Via Bundler (recommended)

Add to your `Gemfile`:

```ruby
gem "rubric", require: false
```

Then run:

```sh
bundle install
bundle exec rubric check
```

### Standalone

```sh
gem install rubric
```

## Usage

### Check for violations

```sh
rubric check                    # lint current directory
rubric check app/               # lint a subdirectory
rubric check app/models/user.rb # lint a single file
```

### Auto-fix safe violations

```sh
rubric check --fix              # fix in place
rubric fmt                      # format (layout + style fixes only)
```

### Migrate from Rubocop

```sh
rubric migrate                  # reads .rubocop.yml, writes rubric.toml
rubric migrate --input .rubocop.yml --output rubric.toml
```

## Configuration

Create `rubric.toml` in your project root:

```toml
[linter]
enabled = true

[formatter]
enabled = true

exclude = ["vendor/**", "db/schema.rb", "spec/fixtures/**"]

[rules]
# Department defaults
"Layout" = { enabled = true }
"Style"  = { enabled = true }
"Lint"   = { enabled = true }

# Rule overrides
"Layout/LineLength"          = { enabled = false }
"Style/FrozenStringLiteralComment" = { enabled = true }
```

## Cops

Rubric ships with 150 cops across three departments:

### Layout (53 cops)

Enforces whitespace, indentation, and formatting rules.

| Cop | Description |
|-----|-------------|
| Layout/TrailingWhitespace | No trailing whitespace on lines |
| Layout/TrailingNewlines | File must end with exactly one newline |
| Layout/IndentationWidth | 2-space indentation |
| Layout/IndentationConsistency | No mixed tabs and spaces |
| Layout/LineLength | Lines must be <= 120 characters |
| Layout/EmptyLines | No consecutive blank lines |
| Layout/SpaceAfterComma | Space required after commas |
| Layout/SpaceAroundOperators | Space required around operators |
| Layout/EndOfLine | LF line endings only (no CRLF) |
| Layout/IndentationStyle | Spaces only, no tabs |
| ... and 43 more |

### Style (49 cops)

Enforces idiomatic Ruby coding patterns.

| Cop | Description |
|-----|-------------|
| Style/FrozenStringLiteralComment | Require `# frozen_string_literal: true` |
| Style/StringLiterals | Prefer single quotes when no interpolation |
| Style/TrailingCommaInArguments | No trailing comma in method args |
| Style/HashSyntax | Use `{ key: value }` not `{ :key => value }` |
| Style/GuardClause | Use guard clauses to reduce nesting |
| Style/RedundantReturn | Omit `return` for last expression |
| Style/NegatedIf | Use `unless` instead of `if !` |
| Style/ReturnNil | Use bare `return` instead of `return nil` |
| Style/AndOr | Use `&&`/`\|\|` not `and`/`or` |
| Style/EmptyMethod | `def foo; end` on one line |
| ... and 39 more |

### Lint (48 cops)

Catches real bugs and suspicious patterns.

| Cop | Description |
|-----|-------------|
| Lint/UnusedMethodArgument | Method argument declared but not used |
| Lint/UselessAssignment | Variable assigned but never read |
| Lint/DuplicateHashKey | Hash has duplicate keys |
| Lint/DuplicateMethods | Method defined twice in same scope |
| Lint/DuplicateRequire | `require` called twice with same argument |
| Lint/UnreachableCode | Code after unconditional `return`/`raise` |
| Lint/FloatOutOfRange | Float literal outside IEEE 754 range |
| Lint/BigDecimalNew | Use `BigDecimal(...)` not `BigDecimal.new(...)` |
| Lint/BooleanSymbol | `:true`/`:false` are symbols, not booleans |
| Lint/RaiseException | Use `StandardError` not bare `Exception` |
| ... and 38 more |

## Adding a New Cop

See [docs/contributing.md](docs/contributing.md) for a step-by-step guide.

## License

MIT. See [gem/LICENSE](gem/LICENSE).
