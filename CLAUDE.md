# Rubric — Claude Context

## What This Is

Rubric is a Ruby linter written in Rust — a Rubocop competitor. ~150 cops implemented. Currently in a false-positive (FP) reduction sprint to match Rubocop's output closely enough to be a viable drop-in replacement.

- **Repo:** https://github.com/nafistiham/rubric (private), cloned to `rubric/` on this machine

---

## Current Status (as of 2026-03-09)

### Milestones
- **M1** ✅ Workspace + core + TrailingWhitespace + CLI
- **M2** ✅ Rayon parallelism + rubric.toml config + 10 cops
- **M3** ✅ Auto-fix engine + `rubric fmt` + 30 cops
- **M4** ✅ `rubric migrate` (.rubocop.yml → rubric.toml) + 75 cops
- **M5** ✅ 150 cops + Criterion benchmarks + gem packaging
- **M6** ✅ README + docs/contributing.md + docs/cops/ + CHANGELOG
- **FP Sprint** 🔄 Session 9 complete (see below)

### Session 9 Benchmark (train set)
| Project | Violations | Previous |
|---------|-----------|---------|
| Puma | 2 | 4 |
| Faker | 51 | 55 |
| Mastodon | 102 | 116 |
| Sidekiq | 473 | 487 |
| Devise | 413 | 424 |

### Next FP Investigation Candidates (Session 10)
| Rule | Project | Count | Priority |
|------|---------|-------|---------|
| `Layout/DefEndAlignment` | faker | ~9 | High |
| `Layout/SpaceAroundOperators` | mastodon+faker | ~30 | High |
| `Lint/AmbiguousRegexpLiteral` | mastodon | ~13 | Medium |
| `Lint/AmbiguousOperator` | mastodon | ~13 | Medium |
| `Style/NestedMethodDefinition` | rails test set | ~297 | High (big win) |

For `NestedMethodDefinition`: `until/while/for/if` one-liners with `; end` are pushing a frame that never gets popped within the same line. Need within-line `end` detection.

---

## Architecture

```
rubric/  (repo root)
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

---

## Setup

```bash
# Build
~/.cargo/bin/cargo build --release -p rubric-cli
# Binary at: target/release/rubric-cli

# Tests
~/.cargo/bin/cargo test --workspace
```

### Test Projects (for FP reduction)
```bash
# Clone into ruby-projects-to-test/ sibling directory
cd ../ruby-projects-to-test
git clone https://github.com/faker-ruby/faker.git faker
git clone https://github.com/mastodon/mastodon.git mastodon
git clone https://github.com/mperham/sidekiq.git sidekiq
git clone https://github.com/heartcombo/devise.git devise
git clone https://github.com/puma/puma.git puma
```

---

## FP Reduction Workflow

```bash
# 1. Build
cargo build --release -p rubric-cli

# 2. Run rubric on train project
exec target/release/rubric-cli check ../ruby-projects-to-test/faker/ 2>/dev/null | sort | uniq -c | sort -rn | head -30

# 3. Run Rubocop on same project
cd ../ruby-projects-to-test/faker && rubocop --format json | jq '.files[].offenses[].cop_name' | sort | uniq -c | sort -rn | head -30

# 4. Find cops where rubric >> rubocop, sample FP cases, fix rule, re-run
```

> **IMPORTANT:** Use `exec target/release/rubric-cli check dir/` pattern in zsh. NOT `rubric-cli check` without `exec`.

---

## Key File Paths

| Path | Purpose |
|------|---------|
| `rubric-rules/src/layout/mod.rs` | Register layout cops |
| `rubric-rules/src/style/mod.rs` | Register style cops |
| `rubric-rules/src/lint/mod.rs` | Register lint cops |
| `rubric-cli/src/main.rs` | CLI binary + rule registration |
| `rubric-core/src/lib.rs` | Rule trait + types |

---

## Git Workflow

```bash
git checkout develop
# ... make changes ...
git add <specific files>
git commit -m "fix(RuleName): description"
git checkout main && git merge develop --ff-only
git push origin main develop
git checkout develop
```

- Never add `Co-Authored-By` trailers to commits
- Commit format: `fix(RuleName): description`
