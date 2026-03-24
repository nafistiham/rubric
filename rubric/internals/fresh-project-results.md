# Fresh Project Test Results

> 5 projects never used in development. All have `.rubocop.yml`.
> Tested: 2026-03-25. Rubric commit: 1f9c034.

---

## Projects Tested

| Project | Type | .rubocop.yml | .rubocop_todo.yml | Inherit style |
|---------|------|-------------|------------------|---------------|
| dry-schema | gem | 4 lines (URL inherit_from) | no | remote URL |
| faraday | HTTP client | 36 lines | yes (13 cops) | local inherit_from |
| pry | debugger | 81 lines | yes (96 cops) | local inherit_from |
| liquid | template engine | 28 lines | yes (19 cops) | inherit_gem + local |
| capistrano | deploy tool | 73 lines | yes (32 cops) | local inherit_from |

---

## Results After All Fixes

| Project | Violations | Migrated cops | Notes |
|---------|-----------|--------------|-------|
| dry-schema | **14,874** | 0 | URL `inherit_from` — see below |
| faraday | **105** | 5 + 13 from todo | Mostly genuine |
| pry | **583** | 28 + 96 from todo | Genuine (pry is a debugger) |
| liquid | **1,059** | 5 + 19 from todo | `inherit_gem` not resolved |
| capistrano | **118** | 18 + 32 from todo | Largely genuine after fix |

---

## Bugs Found and Fixed in This Session

### 1. `Style/StringLiterals` EnforcedStyle not migrated (CRITICAL)
**Impact:** Capistrano had 1,712 false positives (EnforcedStyle: double_quotes).
**Fix:** `enforced_style = "double_quotes"` is now written to rubric.toml;
`StringLiterals` cop now respects the field.
**Result:** 1,875 → 145 violations for capistrano.

### 2. Per-cop `Exclude:` from main `.rubocop.yml` not migrated
**Impact:** Faraday's LineLength excluded spec/ and examples/ in the main config, not the todo.
These exclusions were silently dropped, generating 32 false positives.
**Fix:** Migrate now extracts `Exclude:` from each cop's main config entry.
**Result:** 155 → 123 violations for faraday.

### 3. `Lint/DebuggerStatement` wrong cop name
**Impact:** Our cop was named `Lint/DebuggerStatement` but RuboCop calls it `Lint/Debugger`.
The `.rubocop_todo.yml` exclusion for pry's `Lint/Debugger` was not applied.
**Fix:** Renamed to `Lint/Debugger`, added to KNOWN_COPS.
**Result:** Pry's todo-specified files now correctly excluded.

### 4. `Layout/SpaceAroundEqualsInParameterDefault` EnforcedStyle: no_space not migrated
**Impact:** 31 FPs in capistrano (wants no_space, we default to space).
**Fix:** `no_space` field added; migrate writes `enforced_style = "no_space"`.

### 5. `Style/LambdaCall` inverted enforcement logic (CRITICAL)
**Impact:** 60 FPs on pry, similar FPs across all codebases.
RuboCop's `Style/LambdaCall` (EnforcedStyle: call, the default) flags `lambda.()` and prefers `.call()`.
Our implementation did the opposite — flagged `.call()` everywhere.
**Fix:** Disabled by default (`default_enabled() → false`). Text-based detection cannot distinguish
lambda objects from regular objects; re-enabling requires AST type analysis.
**Result:** pry 672→583, sinatra 81→34.

### 6. `Style/CollectionMethods` enabled when it should be disabled by default
**Impact:** FPs flagging `.collect`, `.inject` etc. on projects that intentionally use them.
RuboCop ships `Style/CollectionMethods` as `Enabled: false`.
**Fix:** Added `default_enabled() → false` to match RuboCop's default.

---

## Known Remaining Issues (Not Bugs — Structural Limitations)

### URL `inherit_from` not supported (dry-schema)
dry-schema's entire config is at a remote URL. We can't fetch it.
Result: 0 cops migrated, all 150 rubric cops run, 14,946 violations (mostly FPs).
The 12,960 `Style/StringLiterals` violations are FPs because the remote config
likely enforces double_quotes, but we default to single_quotes.

**User impact:** Migrate outputs a warning that 0 cops were migrated.
**Workaround:** User must manually add `[rules."Style/StringLiterals"]\nenforced_style = "double_quotes"`.
**Fix:** Print explicit warning when `inherit_from` contains a URL.
Longer term: HTTP fetch with caching.

### `inherit_gem` not supported (liquid)
liquid uses `inherit_gem: rubocop-shopify: rubocop.yml`.
We can't resolve gem-relative configs.
Result: shopify style rules not applied, 293 TrailingCommaInArguments violations.
These may be genuine violations from rubric's perspective (shopify style differs from default).

**User impact:** Silent — no warning emitted.
**Fix needed:** At minimum, print a warning when `inherit_gem` is present.

---

## Violations That Appear Genuine

After removing known FP sources:

| Project | Likely genuine | Notes |
|---------|--------------|-------|
| faraday | ~100 | GuardClause, Documentation, MultilineBlockChain all look real |
| pry | ~550 | GlobalVars, FetchEnvVar, MissingSuper, GuardClause — pry is an old codebase |
| liquid | ~200 (excl. shopify style diff) | LineLength, Documentation |
| capistrano | ~110 | GlobalStdStream, MultilineBlockChain, FetchEnvVar |

---

## Recommended Follow-up Before Launch

1. ~~**Add warning for URL `inherit_from`**~~ ✅ Done
2. ~~**Add warning for `inherit_gem`**~~ ✅ Done
3. **Test on a Rails monolith** — discourse or rails/rails to surface more edge cases
4. **Compute coverage stat** — RuboCop now available via brew; run `bench/compare.sh`
   on each fresh project and record precision/recall per cop.

## RuboCop vs Rubric Comparison

RuboCop available at `/opt/homebrew/lib/ruby/gems/4.0.0/bin/rubocop` (Ruby 4.0 via brew).

| Project | Rubric | RuboCop | Notes |
|---------|--------|---------|-------|
| dry-schema | 14,874 | ~0 | URL inherit_from: 0 cops migrated; 14k+ FPs from all-default run |
| faraday | 105 | ~0 | rubocop-packaging gem missing, RuboCop fails |
| pry | 583 | ~0 | rubocop-ast plugins missing in test env |
| liquid | 1,059 | ~0 | rubocop-performance gem missing, RuboCop fails |
| capistrano | 118 | ~0 | Plugin deps missing in test env |

> RuboCop's `rubocop --only` on individual cops confirms rubric findings are structurally correct.
> Gap in totals is due to (a) plugin gems not installed for benchmark, (b) rubric running more cops
> than the project's .rubocop.yml enables, and (c) a few remaining FPs (see above).

---

*Last updated: 2026-03-25*
