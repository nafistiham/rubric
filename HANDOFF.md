# Rubric — Machine Handoff Doc

Generated: 2026-03-09

## Quick Summary

Rubric is a Ruby linter in Rust — a Rubocop competitor. It has ~150 cops and is in a false-positive (FP) reduction sprint. The goal is to match Rubocop's output closely enough to be a viable drop-in replacement.

---

## Setup on New Machine

### 1. Install Prerequisites

```bash
# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Ruby test projects need Ruby installed, but rubric itself is pure Rust
rustc --version   # stable 1.7x+
```

### 2. Clone the Repo

```bash
mkdir -p ~/Desktop/Learn/Projects/Personal
cd ~/Desktop/Learn/Projects/Personal
git clone https://github.com/nafistiham/rubric.git Rusty
cd Rusty
git checkout develop
```

### 3. Clone Test Projects

```bash
mkdir -p ~/Desktop/Learn/Projects/Personal/ruby-projects-to-test
cd ~/Desktop/Learn/Projects/Personal/ruby-projects-to-test

# Train set (FP reduction targets)
git clone https://github.com/faker-ruby/faker.git faker
git clone https://github.com/mastodon/mastodon.git mastodon
git clone https://github.com/mperham/sidekiq.git sidekiq
git clone https://github.com/heartcombo/devise.git devise
git clone https://github.com/puma/puma.git puma

# Each project needs a rubric.toml — see "rubric.toml Setup" below
```

### 4. Build the Binary

```bash
cd ~/Desktop/Learn/Projects/Personal/Rusty
~/.cargo/bin/cargo build --release -p rubric-cli
# Binary at: target/release/rubric-cli
```

### 5. Create `.claude/settings.local.json`

```bash
cat .claude/settings.local.json  # already in repo, should be fine
```

Content (already in repo):
```json
{ "permissions": { "allow": ["Bash(*)", "WebSearch", ...] } }
```

### 6. Global Claude Setup

Create `~/.claude/CLAUDE.md`:
```markdown
# Global Claude Instructions

## Commits
- Never add `Co-Authored-By` or any co-authorship trailer to commit messages
```

---

## Project State

### Architecture

```
Rusty/  (repo: rubric)
├── rubric-core/    # Rule trait, FileContext, Diagnostic, walker
├── rubric-rules/
│   ├── src/
│   │   ├── layout/       # ~50 cops
│   │   ├── style/        # ~40 cops
│   │   ├── lint/         # ~30 cops
│   │   ├── metrics/      # ~10 cops
│   │   └── ...
│   └── tests/
├── rubric-cli/     # rubric check / rubric fmt / rubric migrate binary
└── docs/
```

### Milestone Status

- **M1** ✅ Workspace + core + TrailingWhitespace + CLI
- **M2** ✅ Rayon parallelism + rubric.toml config + 10 cops
- **M3** ✅ Auto-fix engine + `rubric fmt` + 30 cops
- **M4** ✅ `rubric migrate` (.rubocop.yml → rubric.toml) + 75 cops
- **M5** ✅ 150 cops + Criterion benchmarks + gem packaging
- **M6** ✅ README + docs/contributing.md + docs/cops/ + CHANGELOG

### Current Sprint: FP Reduction (Session 9 complete)

We run Rubric and Rubocop on the same Ruby codebases and fix rules where Rubric reports many more violations than Rubocop.

**Session 9 benchmark (train set):**
| Project | Violations | Previous |
|---------|-----------|---------|
| Puma | 2 | 4 |
| Faker | 51 | 55 |
| Mastodon | 102 | 116 |
| Sidekiq | 473 | 487 |
| Devise | 413 | 424 |

**Session 9 rules fixed (13 total):**
- `FirstHashElementIndentation`: skip YARD doc comment lines
- `SpaceBeforeComment`: skip standalone `##` YARD headers
- `RedundantSplatExpansion`: regex literal detection before `*[`
- `SpaceAfterComma`: multiline `%r{...}` regex state tracking
- `SpaceAroundKeyword`: full rewrite with `in_string: Option<u8>`
- `UselessComparison`: skip when LHS has binary operator prefix
- `CaseIndentation`: heredoc body skip
- `FirstArrayElementIndentation`: heredoc body skip
- `SpaceBeforeBlockBraces`: generalize `%r` skip to all `%x{...}` forms
- `SpaceInsideHashLiteralBraces`: bare `%{...}` skip
- `SpaceInsideArrayLiteralBrackets`: `%w/%W/%i/%I` skip
- `SpaceInsideReferenceBrackets`: same fix
- `NestedMethodDefinition`: one-liner detection for `def foo; body end`

---

## How FP Reduction Works

### Workflow

1. Build release binary: `cargo build --release -p rubric-cli`
2. Run on train project: `target/release/rubric-cli check ruby-projects-to-test/faker/ 2>/dev/null | sort | uniq -c | sort -rn | head -30`
3. Also run Rubocop: `cd ruby-projects-to-test/faker && rubocop --format json | jq '.files[].offenses[].cop_name' | sort | uniq -c | sort -rn | head -30`
4. Find cops where rubric >> rubocop
5. Sample 3–5 FP cases, identify the pattern
6. Fix the rule implementation
7. Re-run to verify count drops

### Invocation Pattern

```bash
# IMPORTANT: use 'exec binary check dir' pattern in zsh
exec target/release/rubric-cli check path/to/project/

# NOT: rubric-cli check (without exec, may have zsh issues)
```

### rubric.toml for Test Projects

Each test project should have a `rubric.toml` that mirrors its `.rubocop.yml`:

```toml
[rules]
disabled_by_default = true

# Enable only what rubocop enables for this project
[rules."Layout/TrailingWhitespace"]
enabled = true

[rules."Style/StringLiterals"]
enabled = true
# enforced_style = "double_quotes"  # match project's rubocop style
```

If a project uses `DisabledByDefault: true` in `.rubocop.yml` (like Puma), set `disabled_by_default = true` in rubric.toml.

---

## Next FP Investigation Candidates (Session 10)

| Rule | Project | Count | Priority |
|------|---------|-------|---------|
| `Layout/DefEndAlignment` | faker | ~9 | High |
| `Layout/SpaceAroundOperators` | mastodon+faker | ~30 | High |
| `Lint/AmbiguousRegexpLiteral` | mastodon | ~13 | Medium |
| `Lint/AmbiguousOperator` | mastodon | ~13 | Medium |
| `Style/NestedMethodDefinition` | rails test set | ~297 | High (big win) |

For `NestedMethodDefinition`: `until/while/for/if` one-liners with `; end` are pushing a frame that never gets popped within the same line. Need within-line `end` detection.

---

## Key Commands

```bash
# Build
~/.cargo/bin/cargo build --release -p rubric-cli

# Run tests
~/.cargo/bin/cargo test --workspace

# Check a project
exec target/release/rubric-cli check path/to/ruby/project/

# Run specific rule tests
~/.cargo/bin/cargo test --test trailing_whitespace_test

# Branch workflow
git checkout develop
# ... make changes ...
git add <specific files>
git commit -m "fix(RuleName): description"
git checkout main && git merge develop --ff-only
git push origin main develop
git checkout develop
```

## Key File Paths

| Path | Purpose |
|------|---------|
| `rubric-rules/src/layout/mod.rs` | Register layout cops |
| `rubric-rules/src/style/mod.rs` | Register style cops |
| `rubric-rules/src/lint/mod.rs` | Register lint cops |
| `rubric-cli/src/main.rs` | CLI binary + rule registration |
| `rubric-core/src/lib.rs` | Rule trait + types |

## Binary Path

```
/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/target/release/rubric-cli
```

On new machine after build:
```
~/Desktop/Learn/Projects/Personal/Rusty/target/release/rubric-cli
```
