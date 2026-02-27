# Rubric — Design Document

**Date:** 2026-02-27
**Status:** Approved
**Author:** Brainstorming session

---

## Overview

Rubric is a Ruby linter and formatter written in Rust. It targets the same audience as Rubocop — Ruby/Rails teams — with a focus on dramatically faster CI times (target: 10–50x over Rubocop).

**Positioning:** Direct Rubocop competitor. Real open-source project. Gem distribution.

---

## Goals

- Replace Rubocop as the default linter/formatter for Ruby projects
- Ship ~150 cops across Style, Layout, and Lint departments in v1
- Provide `rubric migrate` to convert `.rubocop.yml` → `rubric.toml`
- Distribute as a gem with precompiled binaries (no Rust toolchain required)
- Linter and formatter share one unified engine (formatter = linter with auto-fix)

---

## Non-Goals (v1)

- Plugin API (community cop crates) — post v1
- Language server / IDE integration — post v1
- Metrics / Naming / Security departments — post v1 (focus: Style + Layout + Lint)
- Windows support — post v1

---

## Architecture — Approach A: Unified Engine (Ruff Blueprint)

### Cargo Workspace

```
rubric/
├── Cargo.toml                    # workspace
├── rubric-core/                  # Rule trait, Diagnostic, Fix, Walker, Context
│   └── src/
│       ├── rule.rs
│       ├── diagnostic.rs
│       ├── fix.rs
│       ├── walker.rs
│       ├── context.rs
│       └── lib.rs
├── rubric-rules/                 # All ~150 cop implementations
│   └── src/
│       ├── style/                # ~50 cops
│       ├── layout/               # ~60 cops
│       ├── lint/                 # ~40 cops
│       └── lib.rs
└── rubric-cli/                   # Binary — gem entry point
    └── src/
        ├── main.rs
        ├── config.rs             # rubric.toml parsing (serde + toml)
        └── commands/
            ├── check.rs
            ├── fmt.rs
            └── migrate.rs
```

### Parser

Uses `ruby-prism` official Ruby parser via its Rust bindings (`prism` crate on crates.io or direct FFI). ruby-prism is Ruby's official parser as of Ruby 3.3 — correctness is guaranteed.

### Core `Rule` Trait

```rust
/// Every cop implements this trait.
pub trait Rule: Send + Sync {
    /// Rubocop-style identifier: "Style/StringLiterals"
    fn name(&self) -> &'static str;

    /// Which AST node kinds trigger this rule.
    /// Walker only dispatches to rules that opted in — no wasted work.
    fn node_kinds(&self) -> &[NodeKind];

    /// Run the rule against a matched node, returning zero or more diagnostics.
    fn check(&self, ctx: &LintContext, node: &Node) -> Vec<Diagnostic>;

    /// Produce an edit to fix the diagnostic. None = no auto-fix available.
    fn fix(&self, source: &str, diag: &Diagnostic) -> Option<Fix> {
        None
    }
}
```

### Diagnostic + Fix Types

```rust
pub struct Diagnostic {
    pub rule: &'static str,        // "Style/StringLiterals"
    pub message: String,
    pub range: TextRange,          // byte offsets into source
    pub severity: Severity,        // Error | Warning | Info
}

pub struct Fix {
    pub edits: Vec<TextEdit>,      // non-overlapping byte-range replacements
    pub safety: FixSafety,         // Safe | Unsafe
}

pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}
```

### File-Level Parallel Processing (Rayon)

```rust
let diagnostics: Vec<FileDiagnostics> = files
    .par_iter()
    .map(|path| {
        let source = std::fs::read_to_string(path)?;
        let tree = prism::parse(&source);
        let ctx = LintContext::new(path, &source, &config);
        walker::run(&tree, &ctx, &enabled_rules)
    })
    .collect();
```

One parse per file. Single AST walker pass dispatches all enabled rules. This is structurally identical to how Ruff achieves its benchmark numbers.

### Formatter = Linter in Fix Mode

No separate formatter engine. `rubric fmt` is:
1. Run all formatting rules (Style/* + Layout/*)
2. Apply all `FixSafety::Safe` fixes
3. Write corrected source back to disk

This means every formatting rule is also a lint rule — no duplication, no drift between what the linter reports and what the formatter fixes.

---

## CLI Interface

| Command | Description |
|---|---|
| `rubric check [path]` | Lint files, print diagnostics to stdout |
| `rubric check --fix` | Lint and apply safe fixes in place |
| `rubric check --fix-unsafe` | Lint and apply all fixes (safe + unsafe) |
| `rubric fmt [path]` | Format files (applies safe fixes for layout/style rules) |
| `rubric migrate` | Read `.rubocop.yml`, write `rubric.toml` |
| `rubric list-rules` | Show all cops with enabled/disabled status |
| `rubric --version` | Print version |

---

## Configuration — `rubric.toml`

```toml
[linter]
enabled = true

[formatter]
enabled = true

# Glob patterns to exclude
exclude = ["vendor/**", "db/schema.rb"]

[rules]
# Department-level defaults
"Style" = { enabled = true }
"Layout" = { enabled = true }
"Lint" = { enabled = true }

# Rule-level overrides
"Style/StringLiterals"    = { enabled = true, single_quotes = true }
"Layout/IndentationWidth" = { enabled = true, width = 2 }
"Lint/UnusedVariable"     = { enabled = false }
```

The `rubric migrate` command reads `.rubocop.yml` and outputs a `rubric.toml` with equivalent settings. Cops with no Rubric equivalent are commented out with a note.

---

## Gem Distribution

Rubric ships as a family of gems:

| Gem | Contents |
|---|---|
| `rubric-x86_64-linux` | Precompiled Linux x64 binary |
| `rubric-aarch64-linux` | Precompiled Linux ARM64 binary |
| `rubric-x86_64-darwin` | Precompiled macOS x64 binary |
| `rubric-arm64-darwin` | Precompiled macOS ARM64 binary |
| `rubric` | Meta gem — depends on correct platform gem |

**Toolchain:**
- `rb_sys` gem for Rust→Ruby integration
- `rake-compiler` for cross-compilation
- `cross` (Rust) for cross-compiling Linux targets in CI
- GitHub Actions matrix for building platform gems

Users install with:
```ruby
# Gemfile
gem 'rubric', require: false
```
```sh
bundle exec rubric check
```

---

## Test Strategy (TDD)

### Unit Tests — Fixtures

Each cop has a pair of fixture files:

```
rubric-rules/tests/fixtures/
└── style/
    └── string_literals/
        ├── offending.rb      # code that triggers the violation
        └── corrected.rb      # expected output after auto-fix
```

Test macro generates a test for each fixture pair:

```rust
cop_test!(StringLiterals, "style/string_literals");
// Expands to: parse offending.rb → assert diagnostics → apply fix → assert == corrected.rb
```

### Integration Tests

Real Ruby files from popular gems (Rails, Devise, Sidekiq) run through Rubric and Rubocop. Compare:
- Same violations detected (within implemented cop set)
- Auto-fix produces valid Ruby (parse corrected output with ruby-prism, assert no parse errors)

### Benchmark Suite

Benchmark against Rubocop on a large Rails codebase (target: rails/rails). Tracked in CI, regressions block merge.

---

## v1 Cop Scope (~150 cops)

### Style (~50)
StringLiterals, FrozenStringLiteralComment, TrailingCommaInArguments, TrailingCommaInArrayLiteral, SymbolArray, WordArray, HashSyntax, Lambda, Proc, BlockDelimiters, ClassAndModuleChildren, Documentation, EmptyMethod, SingleLineMethods, AccessModifierDeclarations, ConditionalAssignment, GuardClause, IfUnlessModifier, NegatedIf, NegatedWhile, RedundantBegin, RedundantReturn, RedundantSelf, SafeNavigation, TernaryParentheses, ZeroLengthPredicate, And, Or, Not, PercentLiteralDelimiters, PreferredHashMethods, RaiseArgs, ReturnNil, Send, SignalException, StderrPuts, StringConcatenation, StructInheritance, SymbolProc, TrailingUnderscoreVariable, UnlessElse, WhileUntilDo, WhileUntilModifier, YodaCondition, ClassMethods, ModuleFunction, MutableConstant, OptionalArguments, ParallelAssignment, RedundantCondition

### Layout (~60)
IndentationWidth, IndentationConsistency, TrailingWhitespace, TrailingNewlines, EndOfLine, EmptyLines, EmptyLinesAroundClassBody, EmptyLinesAroundModuleBody, EmptyLinesAroundMethodBody, EmptyLinesAroundBlockBody, EmptyLineBetweenDefs, ExtraSpacing, SpaceAroundOperators, SpaceAfterColon, SpaceAfterComma, SpaceAfterMethodName, SpaceAroundBlockParameters, SpaceAroundEqualsInParameterDefault, SpaceAroundKeyword, SpaceBeforeBlockBraces, SpaceBeforeComment, SpaceInLambdaLiteral, SpaceInsideArrayLiteralBrackets, SpaceInsideBlockBraces, SpaceInsideHashLiteralBraces, SpaceInsideParens, SpaceInsideRangeLiteral, SpaceInsideReferenceBrackets, SpaceInsideStringInterpolation, FirstArgumentIndentation, FirstArrayElementIndentation, FirstHashElementIndentation, FirstParameterIndentation, MultilineArrayBraceLayout, MultilineHashBraceLayout, MultilineMethodCallBraceLayout, MultilineMethodCallIndentation, MultilineMethodDefinitionBraceLayout, MultilineOperationIndentation, BlockAlignment, CaseIndentation, ClosingParenthesisIndentation, ConditionPosition, DefEndAlignment, ElseAlignment, EndAlignment, HeredocIndentation, IndentationStyle, LineLength, RescueEnsureAlignment

### Lint (~40)
UnusedVariable, UnusedMethodArgument, UnusedBlockArgument, UselessAssignment, UselessSetterCall, AmbiguousBlockAssociation, AmbiguousOperator, AmbiguousRegexpLiteral, AssignmentInCondition, BigDecimalNew, BooleanSymbol, CircularArgumentReference, ConstantDefinitionInBlock, DeprecatedClassMethods, DisjunctiveAssignmentInConstructor, DuplicateBranch, DuplicateHashKey, DuplicateMethods, DuplicateRequire, EmptyBlock, EmptyConditionalBody, EmptyEnsure, EmptyExpression, EmptyInterpolation, EnsureReturn, FloatOutOfRange, FlipFlop, FormatParameterMismatch, ImplicitStringConcatenation, IneffectiveAccessModifier, MissingCopEnableDirective, MultipleComparison, NestedMethodDefinition, NoReturnInBeginEndBlock, NonLocalExitFromIterator, OrderedMagicComments, ParenthesesAsGroupedExpression, PercentStringArray, RaiseException, RandOne, RedundantSplatExpansion, SafeNavigationConsistency, SelfAssignment, ShadowedException, ShadowingOuterLocalVariable, StructNewOverride, SuppressedException, ToJSON, TopLevelReturnWithArgument, UnderscorePrefixedVariableName, UnreachableCode, UnusedBlockArgument, UriEscapeUnescape, UselessComparison, UselessElseWithoutRescue, Void

---

## Milestones

| Milestone | Deliverable |
|---|---|
| M1 | Cargo workspace setup, ruby-prism integration, `Rule` trait, 1 working cop (`Style/TrailingWhitespace`), test harness |
| M2 | Rayon parallel processing, config parsing (`rubric.toml`), `rubric check` CLI, 10 cops |
| M3 | Auto-fix engine, `rubric fmt`, 30 cops |
| M4 | `rubric migrate` command, 75 cops |
| M5 | 150 cops, benchmark suite, gem packaging (cross-compile CI) |
| M6 | Public release, README, docs, contributor guide |

---

## Key Dependencies

| Crate | Purpose |
|---|---|
| `prism` | ruby-prism Rust bindings (official Ruby parser) |
| `rayon` | Data-parallel file processing |
| `serde` + `toml` | `rubric.toml` config parsing |
| `clap` | CLI argument parsing |
| `anyhow` | Error handling |
| `miette` | Pretty diagnostic output (Rust-native, like `rustc` errors) |
| `glob` | File pattern matching |

---

## Open Questions (post-design)

1. Is `prism` crate available on crates.io, or do we use the C FFI directly via `ruby-prism` gem's headers?
2. `miette` vs custom formatter for diagnostic output — check what Ruff uses.
3. Cross-compilation CI: GitHub Actions + `cross` vs `cargo-zigbuild`.
