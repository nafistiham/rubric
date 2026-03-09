# Rubric — Handoff Document

> Last updated: 2026-03-09

---

## What It Is

**Rubric** is a Ruby linter written in Rust — a direct RuboCop competitor targeting speed (10–100x faster). Currently at **150 cops (rules)** across 6 milestones.

- **Repo:** https://github.com/nafistiham/rubric (public)
- **Binary:** `target/release/rubric-cli`

---

## Current Status

| Milestone | Feature | Status |
|-----------|---------|--------|
| M1 | Workspace + core types + Rule trait + TrailingWhitespace + CLI | ✅ Done |
| M2 | Rayon parallel processing + rubric.toml config + 10 cops | ✅ Done |
| M3 | Auto-fix engine + `rubric fmt` + 30 cops | ✅ Done |
| M4 | `rubric migrate` (.rubocop.yml → rubric.toml) + 75 cops | ✅ Done |
| M5 | 150 cops total + Criterion benchmarks + gem packaging | ✅ Done |
| M6 | README + docs/contributing.md + docs/cops/ + CHANGELOG | ✅ Done |
| **FP Reduction** | Sessions 1–9: False positive fixes across all train projects | 🔄 Session 9 done, session 10 next |

---

## Architecture

```
rubric-core/    ← Rule trait, Diagnostic, FileContext, walker
rubric-rules/   ← all 150 cops, one file per rule
rubric-cli/     ← main.rs, CLI parsing, output formatting
```

Cargo workspace, `rayon` for file-level parallelism.

---

## FP Reduction Sprint

**Goal:** Reduce false positives until rubric ≈ rubocop on real Ruby projects.

**Train set** (`ruby-projects-to-test/`): faker, mastodon, sidekiq, devise, puma

**Test set** (`ruby-projects-benchmark/`): 10 projects with per-project `rubric.toml`

**Current benchmark (post session 9):**
| Project | Rubric violations |
|---------|------------------|
| Puma | 2 |
| Faker | 51 |
| Mastodon | 102 |
| Sidekiq | 473 |
| Devise | 413 |

**Test set (apples-to-apples vs rubocop):**
| Project | Rubric | Rubocop |
|---------|--------|---------|
| sinatra | 187 | 69 |
| rspec-core | 938 | 55 |
| activeadmin | 43 | 0 |
| jekyll | 957 | 24 |
| rails | 1838 | 3 |

---

## Session 9 — Last Completed (13 FP fixes)

`FirstHashElementIndentation`, `SpaceBeforeComment`, `RedundantSplatExpansion`, `SpaceAfterComma` (multiline `%r`), `SpaceAroundKeyword` (XPath strings), `UselessComparison`, `CaseIndentation` (heredoc), `FirstArrayElementIndentation` (heredoc), `SpaceBeforeBlockBraces` (all `%x{}`), `SpaceInsideHashLiteralBraces` (`%{}`), `SpaceInsideArrayLiteralBrackets` (`%w/%i`), `SpaceInsideReferenceBrackets`, `NestedMethodDefinition` (` end` suffix)

All fixes on **develop branch** — not yet merged to main.

---

## Session 10 — Next Candidates

- `Layout/DefEndAlignment` faker (~9 residual FPs)
- `Layout/SpaceAroundOperators` mastodon+faker
- `Lint/AmbiguousRegexpLiteral` mastodon
- `Lint/AmbiguousOperator` mastodon
- `NestedMethodDefinition` rails — `until/while/for/if` one-liners pushing extra frames

---

## Key Paths

```
Binary:       rubric/target/release/rubric-cli
Train set:    /ruby-projects-to-test/
Test set:     /ruby-projects-benchmark/
Invocation:   exec binary check dir   (NOT: binary check dir in zsh)
```

## Git Workflow

```
main ← production ← develop ← feature/*
```
- Rebase only, no squash
- `gh pr merge --rebase` → force-push develop to production and main

---

## Setup on New Machine

```bash
cd Rusty/rubric
cargo build --release
# binary at target/release/rubric-cli
```

---

## What To Do Next

1. **Merge session 3–9 FP fixes** from develop → production → main (pending PR review)
2. **Session 10** — target the FP candidates listed above
3. Keep reducing test set gaps (rails 1838 → closer to rubocop's 3)
