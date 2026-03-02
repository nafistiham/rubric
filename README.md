# Rubric

> A Ruby linter and formatter, written in Rust.

[![Build](https://github.com/nafistiham/rubric/actions/workflows/release.yml/badge.svg)](https://github.com/nafistiham/rubric/actions)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](https://github.com/nafistiham/rubric/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](gem/LICENSE)

Rubric is a drop-in replacement for Rubocop — same cop names, same `.rubocop.yml` migration path — but built in Rust for dramatically faster CI times.

```
$ rubric check app/

app/models/user.rb:14:1  [W] Layout/TrailingWhitespace     Trailing whitespace detected.
app/models/user.rb:27:5  [W] Style/GuardClause              Use a guard clause instead of wrapping the code inside a conditional expression.
app/services/payment.rb:3:1  [W] Style/FrozenStringLiteralComment  Missing frozen string literal comment.

3 violations found in 0.12s across 47 files.
```

---

## Why Rubric?

Rubocop is the standard. It's also slow — seconds on small projects, minutes on large ones. Rubric runs the same checks in milliseconds by processing files in parallel across all CPU cores and avoiding Ruby's startup overhead entirely.

| Project size | Rubocop | Rubric | Speedup |
|---|---|---|---|
| 100 files | ~8s | ~180ms | **44×** |
| 500 files | ~35s | ~700ms | **50×** |
| 2,000 files | ~140s | ~3s | **47×** |

*Measured on Apple M2, 8 cores. Your numbers will vary, but the gap won't.*

---

## Installation

### Bundler (recommended)

```ruby
# Gemfile
gem 'rubric-linter', require: false
```

```sh
bundle install
bundle exec rubric check
```

### Standalone

```sh
gem install rubric-linter
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

# Migrate from Rubocop
rubric migrate                   # .rubocop.yml → rubric.toml
```

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

Reads `.rubocop.yml` and writes `rubric.toml`. Cops that Rubric implements are mapped directly. Cops that Rubric doesn't implement yet are preserved as comments so you don't lose configuration.

```toml
# rubric.toml (generated)
[rules."Style/StringLiterals"]
enabled = true

# UNKNOWN: Metrics/PerceivedComplexity (not yet implemented in Rubric)
# UNKNOWN: Naming/MethodParameterName (not yet implemented in Rubric)
```

---

## Contributing

Adding a new cop takes about 15 minutes and follows a strict TDD workflow — fixture files first, failing test, then implementation. See [`docs/contributing.md`](docs/contributing.md) for the full walkthrough.

---

## License

MIT — see [`gem/LICENSE`](gem/LICENSE).
