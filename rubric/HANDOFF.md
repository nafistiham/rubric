# Rubric — Handoff Context

> Use this doc to resume work on a new machine quickly.

---

## Project Overview

**Rubric** is a Ruby linter written in Rust — a RuboCop competitor aiming for RuboCop's rule coverage at Rust-level speed.

- **Repo:** https://github.com/nafistiham/rubric (public)
- **Language:** Rust (Cargo workspace)
- **Binary:** `rubric-cli` — invoke as `rubric check <path>` or `rubric fmt <path>`
- **Rules:** 150 cops across Layout, Lint, Style categories

---

## Machine Setup

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ruby (for comparison tests)
brew install ruby  # or rbenv

# RuboCop (for benchmark comparison)
gem install rubocop  # should be 1.85.x
```

### Clone and Build

```bash
git clone https://github.com/nafistiham/rubric.git
cd rubric
git checkout develop

cargo build --release --bin rubric-cli

# Binary ends up at:
# target/release/rubric-cli
```

### Run Tests

```bash
cargo test
```

---

## Repository Layout

```
rubric/
├── rubric-core/         # Rule trait, LintContext, Diagnostic types, AST walker
├── rubric-rules/        # 150 cop implementations + tests + fixtures
│   ├── src/layout/
│   ├── src/lint/
│   ├── src/style/
│   ├── tests/           # one test file per rule
│   └── tests/fixtures/  # offending.rb / clean.rb per rule
├── rubric-cli/          # CLI binary (check, fmt, migrate commands)
│   └── src/
│       ├── main.rs
│       ├── config.rs    # rubric.toml parsing + per-cop enable/disable
│       ├── runner.rs    # parallel file linting with rayon
│       └── commands/
├── gem/                 # rubric-linter gem packaging
├── internals/           # Research docs (bug analysis, FP root causes)
└── HANDOFF.md           # This file
```

---

## Key Commands

```bash
# Build release binary
cargo build --release --bin rubric-cli

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p rubric-rules

# Lint a project
./target/release/rubric-cli check /path/to/ruby-project

# Lint with auto-fix
./target/release/rubric-cli fmt /path/to/ruby-project

# Migrate rubocop config
./target/release/rubric-cli migrate /path/to/.rubocop.yml

# Run criterion benchmarks
cargo bench
```

---

## Config System

`rubric.toml` (place in the project root being linted):

```toml
exclude = ["tmp/*", "vendor/*"]   # * matches any chars including /

[linter]
enabled = true
disabled_by_default = true        # disables all cops unless explicitly enabled

[formatter]
enabled = true

[rules."Layout/LineLength"]
enabled = true

[rules."Style/StringLiterals"]
enabled = false
```

Key notes:
- `disabled_by_default = true` → only explicitly listed cops run
- Glob patterns: use `test/*` not `test/**/*` (our `*` already matches `/`)
- Config is loaded from the target dir first, falls back to CWD

---

## Branching Model

```
main ← production ← develop ← feature/*
```

- All work happens on `develop`
- Merge chain: develop → production → main (rebase, no squash)
- Never commit directly to main or production

### Merge Procedure

```bash
git checkout production && git rebase develop && git push origin production
git checkout main && git rebase production && git push origin main
git checkout develop
```

---

## Current State (Session 9 complete)

### Train Set Benchmark (ruby-projects-to-test/)

| Project | Violations |
|---------|-----------|
| faker   | 51        |
| mastodon | 102      |
| sidekiq | 473       |
| devise  | 413       |
| puma    | 2         |

### Test Set Benchmark (ruby-projects-benchmark/)

Apples-to-apples vs rubocop: each project has `rubric.toml` with `disabled_by_default = true` + only cops rubocop enables (intersection of rubric's 150 cops and rubocop's enabled set), minus cops with mismatched `EnforcedStyle`.

| Project     | Rubric | Rubocop | Ratio |
|-------------|--------|---------|-------|
| sinatra     | 187    | 69      | 2.7x  |
| rspec-core  | 938    | 55      | 17.1x |
| activeadmin | 43     | 0       | —     |
| jekyll      | 957    | 24      | 39.9x |
| rails       | 1838   | 3       | 612x  |

### Session 9 FP Fixes (all on develop)

1. `FirstHashElementIndentation` — skip YARD doc comment lines
2. `SpaceBeforeComment` — skip standalone `##` YARD headers
3. `RedundantSplatExpansion` — skip regex literals before `*[` check
4. `SpaceAfterComma` — cross-line `%r{...}` multiline regex state
5. `SpaceAroundKeyword` — full char-loop rewrite; skip `not(` in strings
6. `UselessComparison` — skip compound binary-op LHS
7. `CaseIndentation` — skip heredoc bodies
8. `FirstArrayElementIndentation` — skip heredoc bodies
9. `SpaceBeforeBlockBraces` — skip all `%x{...}` percent-literal forms
10. `SpaceInsideHashLiteralBraces` — skip bare `%{...}` string literals
11. `SpaceInsideArrayLiteralBrackets` — skip `%w[...]`/`%i[...]` delimiters
12. `SpaceInsideReferenceBrackets` — skip `%w[...]`/`%i[...]` delimiters
13. `NestedMethodDefinition` — detect one-liners ending with ` end` (not just `; end`)

---

## Session 10 — Priority Task List

### High Priority (biggest FP count remaining)

1. **`NestedMethodDefinition` — keyword one-liner frames** (rails: ~297 FPs)
   - `until/while/for/if` with `; end` on same line push a frame but the embedded `end` isn't consumed
   - Example: `until token = scan || @scanner.eos?; end`
   - Fix: `is_one_liner_def` logic also needs to apply to non-def one-liner constructs, OR detect same-line `end` at frame-push time

2. **`DuplicateRequire`** (rails: ~277 FPs)
   - Need to sample: possibly cross-file false triggers or conditional requires
   - Check if `require_relative` + `require` of same file is being double-counted

3. **`DuplicateMethods`** (rails: ~147 FPs)
   - DSL method patterns (ActiveRecord callbacks, route helpers, etc.) likely triggering
   - Need to sample actual violations

4. **`HashAlignment`** (jekyll: ~371 FPs)
   - jekyll uses `EnforcedLastArgumentHashStyle: always_inspect` — style config mismatch
   - Option: exclude `HashAlignment` from jekyll's `rubric.toml`

### Medium Priority

5. **`DefEndAlignment`** (faker: ~9 FPs) — residual after session 8 fix, need to sample
6. **`SpaceAroundOperators`** (mastodon+faker) — some compound-assign patterns still FPing
7. **`AmbiguousRegexpLiteral`** (mastodon: ~13) — check for FPs
8. **`AmbiguousOperator`** (mastodon: ~13) — check for FPs

### Architecture (long-term)

The `internals/` folder documents root-cause analysis. The core issue:
rules needing structural knowledge of Ruby (block vs hash, regex vs division) are implemented as text scanners. Rewriting key rules as `check_node` (AST-visitor) will eliminate entire classes of FPs. The infrastructure exists — `rule.rs` has `check_node()`, `walker.rs` dispatches all prism node types.

Priority rewrite candidates (from `internals/index.md`):
- `DuplicateHashKey` → `check_node` visiting `HashNode`
- `SpaceAroundOperators` → `check_node` visiting `BinaryNode`
- `UnreachableCode` → `check_node` visiting return/raise/break + if_mod guard

---

## Benchmark Projects Location

```
~/Desktop/Learn/Projects/Personal/ruby-projects-to-test/   # train set
~/Desktop/Learn/Projects/Personal/ruby-projects-benchmark/ # test set (with rubric.toml per project)
```

These are local clones — re-clone on the new machine:

```bash
# Train set projects
cd ~/Desktop/Learn/Projects/Personal/ruby-projects-to-test
# faker, mastodon, sidekiq, devise, puma — clone from github

# Test set projects (also have rubric.toml files in the repo at ruby-projects-benchmark/)
# sinatra, rspec-core, activeadmin, jekyll, rails — clone from github
```

The `rubric.toml` files for test set projects are in:
`/ruby-projects-benchmark/<project>/rubric.toml` — but that directory is NOT in this repo.
You need to recreate them or copy them from the old machine.

**rubric.toml contents per project (reconstruct if needed):**
- All start with `disabled_by_default = true`
- Copy the cop list from the old `rubric.toml` files
- Key exclusions per project:
  - sinatra: `test/*`, `rack-protection/*`, `sinatra-contrib/*`, `vendor/bundle/*`
  - rspec-core: `lib/rspec/core/backport_random.rb`, `benchmarks/*`
  - jekyll: `benchmark/*`, `bin/*`, `exe/*`, `script/*`, `tmp/*`, `vendor/*`, `features/*`, `docs/*`, `site/*`
  - rails: `tmp/*`, `*/tmp/*`, `*/templates/*`, `*/vendor/*`, `actionmailbox/test/dummy/*`, `activestorage/test/dummy/*`, `actiontext/test/dummy/*`
  - activeadmin: `.git/*`, `.github/*`, `bin/*`, `gemfiles/*/vendor/*`, `node_modules/*`, `tmp/*`, `vendor/*`

---

## Memory / Claude Context

Claude's persistent memory for this project lives at:
`~/.claude/projects/-Users-md-tihami-Desktop-Learn-Projects-Personal-Rusty/memory/MEMORY.md`

On the new machine, this path won't exist yet. The HANDOFF.md you're reading now contains all context needed to continue. After your first session, Claude will auto-populate memory.

---

## No Secrets / Env Files Required

This project has no API keys, no `.env` files, no secrets. Everything is open source.
The only credential needed is SSH/HTTPS access to the GitHub repo.
