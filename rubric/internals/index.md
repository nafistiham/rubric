# Rubric Internals — Bug Research Index

**Created:** 2026-03-02
**Source data:** Faker (553 files) and Mastodon (1,195 files) benchmark reports
**Scope:** Root-cause analysis of every confirmed false positive and missed detection

This folder documents the internal bugs identified through real-world linting of two major Ruby projects. No code has been changed. All documents are research only.

---

## The Architectural Theme

Every bug found reduces to **one root cause**: rules that need structural knowledge of Ruby are implemented as pure byte/text scanners (`check_source`) instead of AST-aware node visitors (`check_node`).

The infrastructure to fix this **already exists**:
- `rubric-core/src/rule.rs` defines `check_node()` and `node_kinds()` on the `Rule` trait
- `rubric-core/src/walker.rs` implements a complete `ruby-prism` AST walker that already dispatches `BlockNode`, `HashNode`, `CallNode`, `IfNode`, `ReturnNode`, `RegularExpressionNode`, and every other node type

Rules are using `check_source` as a shortcut that works for simple patterns (trailing whitespace, frozen string comment) but breaks badly on anything that requires knowing what syntactic construct surrounds a token.

RuboCop does not have this problem because it was built AST-first. Every cop in RuboCop receives a fully typed AST node. It cannot confuse a regex `/` with a division operator because those are different node types at the point of dispatch.

---

## Bug Documents

### 1. [Lint/UnreachableCode + Style/NegatedIf](./unreachable-code-and-negated-if.md)

**Benchmark impact:** 40 UnreachableCode false positives on Mastodon; NegatedIf missed the injected modifier-`if !` in Faker

**Root causes:**

| Bug | Location | Root cause |
|-----|----------|------------|
| UnreachableCode FP | `unreachable_code.rs:30–35` | `trimmed.starts_with("return")` is true for `return if condition` — no trailing-modifier check exists. Every subsequent sibling line at the same indent is then unconditionally flagged. |
| NegatedIf miss | `negated_if.rs:16` | `if !trimmed.starts_with("if ")` skips any line where `if` is not the first token. Modifier position (`foo if !bar`) is never checked. |

**Fix direction:** Rewrite both as `check_node` rules. UnreachableCode should visit `return`/`raise`/`break`/`next` nodes and check whether they are wrapped in an `if_mod`/`unless_mod` node — if so, they are conditional and do not make subsequent statements unreachable. NegatedIf should visit all `if_mod` and `if` nodes and check whether the condition is a `not` node.

→ See [unreachable-code-and-negated-if.md](./unreachable-code-and-negated-if.md) for exact line references and algorithmic detail.

---

### 2. [Lint/DuplicateHashKey](./duplicate-hash-key.md)

**Benchmark impact:** String key duplicates undetected (Mastodon); 7 false positives on nested sub-hash keys (Mastodon)

**Root causes:**

| Bug | Location | Root cause |
|-----|----------|------------|
| String keys not detected | `duplicate_hash_key.rs:31` | Entry gate `b.is_ascii_alphabetic() \|\| b == b'_'` only enters key detection for symbol-style tokens. `'string'` and `"string"` keys begin with `'` or `"` — silently skipped. The comment at lines 21–22 says `"word" =>` is handled; there is no such code. |
| False positives on nested keys | `duplicate_hash_key.rs:25` | `seen: HashSet<String>` is reset per source line, not per hash literal. No bracket-depth tracking exists. `mention: [mention: :status]` causes the scanner to find `mention:` twice on one line and fire. |

**Fix direction:** Rewrite as `check_node` visiting `HashNode` and `KeywordHashNode`. For each hash node, collect all direct child key nodes (no recursion into sub-hashes). Compare by the key's literal string value regardless of whether it is a symbol, string, or integer node.

→ See [duplicate-hash-key.md](./duplicate-hash-key.md) for exact line references and the full code walkthrough.

---

### 3. [Layout/SpaceAroundOperators](./space-around-operators.md)

**Benchmark impact:** 47 false positives across 5 Mastodon files; 43 false positives in Faker

**Root causes (5 categories):**

| Category | FP source | Root cause |
|----------|-----------|------------|
| `\|\|=` compound assign | `space_around_operators.rs:37` | `\|\|` is matched as a two-byte operator. `bytes[j+2]` is `=`, not a space, so it fires. No three-character lookahead exists. |
| `**` keyword splat | `space_around_operators.rs:38` | `**` is in the two-char match list and checked for surrounding spaces. No positional context (preceded by `(` or `,`) distinguishes splat prefix from exponentiation. |
| Regex metacharacters | `space_around_operators.rs` (single-char loop) | No regex literal tracking state. `/regex/` delimiters are not parsed — characters inside `/.../` and `%r{...}` are scanned as code. `-`, `+`, `*` inside character classes fire freely. |
| `%r{...}` content | Same | `%r` not recognised at all as a literal delimiter. |
| Alignment `=` in groups | `space_around_operators.rs:116` | The unary guard only skips when `prev` is a symbol character. Double-space alignment (e.g. `@text  = ...`) makes `prev` a space, so the `=` is checked — but `prev_ok` and `next_ok` are both true and no diagnostic fires. The reported FPs on aligned blocks come from other operators on those same lines. |

**The misleading comment at line 35** says "Skip compound operators" but the block beneath it checks them, not skips them. Only `->` is genuinely skipped (lines 60–63).

**Fix direction:** The correct approach is to rewrite as a `check_node` rule. Relevant AST nodes: `BinaryNode` (binary operators), `LocalVariableOrWriteNode`/`InstanceVariableOrWriteNode` (compound assignment — skip these entirely), `KeywordRestParameterNode` (double splat in params — skip), `RegularExpressionNode` (ignore contents). As a shorter-term fix that stays in `check_source`: add a three-char lookahead to skip `||=`/`&&=`; track regex delimiters with an `in_regex: bool` state parallel to `in_string`; and add a "preceded by `(` or `,`" guard before flagging `**`.

→ See [space-around-operators.md](./space-around-operators.md) for the full operator table analysis and per-category evidence.

---

### 4. [Layout/MultilineMethodCallBraceLayout + LeadingCommentSpace + SpaceInsideBlockBraces](./layout-false-positives.md)

**Benchmark impact:** 172 MultilineMethodCallBraceLayout FPs (Faker+Mastodon); 96 LeadingCommentSpace FPs (Faker); 36 SpaceInsideBlockBraces FPs (Mastodon)

**Root causes:**

| Rule | FP count | Root cause |
|------|----------|------------|
| `MultilineMethodCallBraceLayout` | 172 | Line 19: fires on any line ending in `)` that is not `)`-only. Line 26: scans backwards for any `(` with no bracket-balance tracking. Hard-codes `new_line` style — requires `)` alone on its own line — but RuboCop's default is `symmetrical` where `)` after the last arg is valid. |
| `LeadingCommentSpace` | 96 | Line 38: checks `bytes[1] != b' '`. For `##`, `bytes[1]` is `b'#'` — fires. The explicit allow-list covers `#!`, `# encoding:`, `# frozen_string_literal:` but **not `##`**. Auto-fix would corrupt YARD docs by rewriting `##` → `# #`. |
| `SpaceInsideBlockBraces` | 36 | Hash/block classifier at lines 56–59 checks preceding chars (`=`, `,`, `(`, `[`, `{`) but **missing `:`** — `key: { ... }` is classified as a block brace. The `}` check at lines 73–84 performs zero hash/block discrimination — every `}` without a preceding space is flagged unconditionally. |
| `IndentationWidth` (supporting) | 49 (Mastodon) | Line 27: `spaces % 2 != 0` applied to every non-empty line including continuation lines aligned to method-call arguments. 11-space `delegate` alignment triggers it. RuboCop only checks structural indentation inside keyword blocks. |

**Fix direction — quick wins first:**
1. `LeadingCommentSpace`: add `if trimmed.starts_with("##") { continue; }` before the space check. Two lines. Zero risk.
2. `SpaceInsideBlockBraces`: add `:` to the hash-predecessor set (line 59), and add the same hash-detection logic to the `}` check path.
3. `MultilineMethodCallBraceLayout`: implement bracket-balance tracking to find the correct matching `(`, then allow closing `)` on the same line as the last argument when the opening `(` is on the same line as the method name (`symmetrical` style).
4. `IndentationWidth`: skip lines whose indentation aligns to a method-call continuation (i.e., the previous non-empty line ends with `,` or `(` or `\`).

→ See [layout-false-positives.md](./layout-false-positives.md) for the exact source line analysis.

---

## Priority Order for Fixes

Ordered by impact (false positives eliminated) vs. implementation effort:

| Priority | Rule | Action | Estimated FP reduction |
|----------|------|---------|----------------------|
| 1 | `LeadingCommentSpace` | Add `##` exemption (2 lines) | 96 Faker FPs |
| 2 | `UnreachableCode` | Add trailing-modifier guard OR rewrite as `check_node` | 40+ Mastodon FPs |
| 3 | `SpaceAroundOperators` — `\|\|=` + `**` | Add 3-char lookahead + splat position guard | ~30 FPs |
| 4 | `SpaceInsideBlockBraces` | Add `:` to hash-predecessor + discriminate `}` | 36 Mastodon FPs |
| 5 | `MultilineMethodCallBraceLayout` | Add bracket-balance + symmetrical style | 172 FPs total |
| 6 | `DuplicateHashKey` | Rewrite as `check_node` (HashNode visitor) | 7 FPs + adds string key detection |
| 7 | `NegatedIf` | Add internal `if !` search OR rewrite as `check_node` | Missed detection fix |
| 8 | `IndentationWidth` | Skip continuation lines | 49 Mastodon FPs |
| 9 | `SpaceAroundOperators` — regex | Add regex delimiter tracking state | ~10 FPs |

---

## How RuboCop Compares

RuboCop is immune to all of these failures for one structural reason: **every cop is dispatched on a specific AST node type**. When RuboCop processes `{ key: value }`, it receives a `hash` node. When it processes `array.each { |x| x }`, it receives a `block` node. These are different dispatch targets — there is no ambiguity. A cop registered for `block` nodes never sees hash nodes.

Rubric has the same infrastructure. The `rule.rs` trait has `check_node()`. The `walker.rs` dispatches every prism node type. The investment needed is rewriting the affected `check_source` implementations to `check_node` implementations — not adding new infrastructure.

---

## Files in This Folder

| File | Contents |
|------|---------|
| `index.md` | This file — master index with summaries |
| `unreachable-code-and-negated-if.md` | Deep analysis of UnreachableCode FPs and NegatedIf miss |
| `duplicate-hash-key.md` | Deep analysis of string-key miss and nested sub-hash FPs |
| `space-around-operators.md` | Deep analysis of 5 categories of SpaceAroundOperators FPs |
| `layout-false-positives.md` | Deep analysis of MultilineMethodCallBraceLayout, LeadingCommentSpace, SpaceInsideBlockBraces, IndentationWidth |
