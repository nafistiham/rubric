# Rubric

> A Ruby linter and formatter, written in Rust.

[![Build](https://github.com/nafistiham/rubric/actions/workflows/release.yml/badge.svg)](https://github.com/nafistiham/rubric/actions)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](https://github.com/nafistiham/rubric/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](gem/LICENSE)

Rubric is a fast Ruby linter written in Rust — 150 cops, same naming as RuboCop, with a migration path from `.rubocop.yml`. Inspired by what Ruff did for Python.

```
$ rubric check app/

app/models/user.rb:14:1  [W] Layout/TrailingWhitespace     Trailing whitespace detected.
app/models/user.rb:27:5  [W] Style/GuardClause              Use a guard clause instead of wrapping the code inside a conditional expression.
app/services/payment.rb:3:1  [W] Style/FrozenStringLiteralComment  Missing frozen string literal comment.

3 violations found in 0.12s across 47 files.
```

---

## Why Rubric?

RuboCop is the standard. It's also slow — seconds on small projects, minutes on large ones. Rubric processes files in parallel across all CPU cores and skips Ruby's startup overhead entirely.

Measured on Apple M2 (arm64), RuboCop 1.85.1, cold start with one warmup run:

| Project | Files | RuboCop | Rubric | Speedup |
|---------|-------|---------|--------|---------|
| sinatra | 147 | 809 ms | 68 ms | **11.9×** |
| jekyll | 160 | 615 ms | 83 ms | **7.4×** |
| rspec-core | 233 | 786 ms | 100 ms | **7.9×** |
| activeadmin | 278 | 604 ms | 45 ms | **13.4×** |

**Average: ~10× faster on cold start** (no warmup, no daemon). Each project uses its own config (rubric.toml mirroring .rubocop.yml).

RuboCop's `--server` mode (persistent daemon) reduces its cold-start overhead but still runs ~3× slower than Rubric on the same projects. In CI — where `--server` has no effect — the full 10× advantage applies.

---

## Installation

### Bundler (recommended)

```ruby
# Gemfile
gem 'rubric', require: false
```

```sh
bundle install
bundle exec rubric check
```

### Standalone

```sh
gem install rubric
```

No Rust toolchain required. The gem ships a precompiled binary for your platform.

---

## Commands

```sh
# Lint
rubric check                     # current directory
rubric check app/                # specific path
rubric check app/models/user.rb  # single file

# Fix
rubric check --fix               # apply safe auto-fixes in place
rubric fmt                       # format only (Layout + Style rules)

# Adopt incrementally (generate a todo baseline)
rubric todo                      # write .rubric_todo.toml — suppress pre-existing violations
rubric check                     # now passes; new violations still surface
rubric check --regenerate-todo   # shrink the baseline as you fix things
rubric check --ignore-todo       # see the full picture including suppressed violations

# Migrate from Rubocop
rubric migrate                   # .rubocop.yml → rubric.toml
```

For a full adoption walkthrough, see [docs/todo-baseline.md](docs/todo-baseline.md).

---

## Configuration

Rubric is configured via `rubric.toml` in your project root. Running `rubric migrate` generates one from your existing `.rubocop.yml`.

```toml
[linter]
enabled = true

[formatter]
enabled = true

exclude = ["vendor/**", "db/schema.rb", "spec/fixtures/**"]

[rules]
"Layout" = { enabled = true }
"Style"  = { enabled = true }
"Lint"   = { enabled = true }

# Per-rule overrides
"Layout/LineLength"                = { enabled = false }
"Style/FrozenStringLiteralComment" = { enabled = true }
```

---

## Cops

Rubric ships with **150 cops** across three departments. The full list is in [`docs/cops/README.md`](docs/cops/README.md).

### Layout (53)

Whitespace, indentation, and formatting. All auto-fixable.

`TrailingWhitespace` · `IndentationWidth` · `LineLength` · `EmptyLines` · `SpaceAfterComma` · `SpaceAroundOperators` · `EndOfLine` · `IndentationStyle` · `EmptyLinesAroundClassBody` · `EmptyLinesAroundMethodBody` · `SpaceBeforeBlockBraces` · `SpaceInsideHashLiteralBraces` · `MultilineMethodCallIndentation` · `CaseIndentation` · `EndAlignment` · [and 38 more →](docs/cops/README.md#layout)

### Style (49)

Idiomatic Ruby patterns. Safe auto-fixes where possible.

`FrozenStringLiteralComment` · `StringLiterals` · `HashSyntax` · `GuardClause` · `RedundantReturn` · `NegatedIf` · `UnlessElse` · `AndOr` · `SymbolArray` · `WordArray` · `TrailingCommaInArguments` · `SafeNavigation` · `RedundantSelf` · `EmptyMethod` · `ReturnNil` · [and 34 more →](docs/cops/README.md#style)

### Lint (48)

Real bugs and suspicious patterns. No auto-fix — these need human review.

`UnusedMethodArgument` · `UselessAssignment` · `DuplicateHashKey` · `DuplicateMethods` · `DuplicateRequire` · `UnreachableCode` · `FloatOutOfRange` · `BooleanSymbol` · `RaiseException` · `AssignmentInCondition` · `SuppressedException` · `NonLocalExitFromIterator` · `ShadowingOuterLocalVariable` · [and 35 more →](docs/cops/README.md#lint)

---

## Migrating from Rubocop

```sh
rubric migrate
```

Reads `.rubocop.yml` and writes `rubric.toml`. If `.rubocop_todo.yml` exists in the same directory, it is merged automatically — per-cop `Exclude:` lists and `Enabled: false` entries carry over, so your existing violation suppressions are preserved.

Cops that Rubric implements are mapped directly. Cops that Rubric doesn't implement yet are preserved as comments so you don't lose configuration.

```toml
# rubric.toml (generated)
[rules."Style/StringLiterals"]
enabled = true

# Exclude list from .rubocop_todo.yml
[rules."Layout/TrailingWhitespace"]
enabled = true
exclude = ["app/legacy/old_file.rb", "db/schema.rb"]

# UNKNOWN: Metrics/PerceivedComplexity (not yet implemented in Rubric)
# UNKNOWN: Naming/MethodParameterName (not yet implemented in Rubric)
```

---

## Current Limitations

Rubric is actively developed. Before adopting it, know what it is and isn't today:

- **150 of ~450 RuboCop cops implemented.** The cops covered are the most commonly triggered ones. See [`docs/cops/README.md`](docs/cops/README.md) for the full list. Cops not yet implemented are silently skipped.
- **No plugin system.** `rubocop-rails`, `rubocop-rspec`, `rubocop-performance`, and similar extensions are not supported yet. Projects that rely on these will miss those checks.
- **`rubric migrate` generates a starting config, not a perfect one.** It maps cops Rubric implements, merges your `.rubocop_todo.yml` suppressions, and comments out unimplemented cops. `EnforcedStyle` options may need manual tuning. Treat the output as a first draft.
- **Scope-dependent rules are disabled by default.** Cops like `Lint/UnusedMethodArgument` require full Ruby scope analysis. They are implemented but off by default until an AST-backed version ships, to avoid false positives on common patterns.

---

## Contributing

Adding a new cop takes about 15 minutes and follows a strict TDD workflow — fixture files first, failing test, then implementation. See [`docs/contributing.md`](docs/contributing.md) for the full walkthrough.

---

## License

MIT — see [`gem/LICENSE`](gem/LICENSE).
