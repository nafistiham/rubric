# Bug Research: Lint/DuplicateHashKey

**Date:** 2026-03-02
**Rule file:** `rubric-rules/src/lint/duplicate_hash_key.rs`
**Test file:** `rubric-rules/tests/duplicate_hash_key_test.rs`

---

## Summary of Both Bugs

| # | Bug | Symptom |
|---|-----|---------|
| 1 | String key duplicates not detected | `{ 'Mention' => :mention, 'Mention' => :duplicate_mention }` produces zero violations |
| 2 | False positives on nested sub-hash keys | `TARGET_STATUS_INCLUDES_BY_TYPE` reports 7 spurious violations because inner sub-hash keys inside `[]` arrays are counted against the outer hash's key set |

Both bugs share a single architectural root cause: the rule is implemented as a **line-by-line text scanner** rather than an **AST-aware hash-scope visitor**. It has no concept of hash boundaries, nesting depth, or key types other than `word:` symbol syntax.

---

## How the Current Code Works

### Source: `rubric-rules/src/lint/duplicate_hash_key.rs`

```rust
fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = &ctx.lines;

    for (i, line) in lines.iter().enumerate() {         // line 15: iterate every line
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }       // line 17: skip comment lines

        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut seen: HashSet<String> = HashSet::new(); // line 25: NEW set per line
        let mut j = 0;

        while j < len {
            let b = bytes[j];
            if b.is_ascii_alphabetic() || b == b'_' {   // line 31: word-start chars only
                let key_start = j;
                while j < len && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                let key_end = j;

                if j < len && bytes[j] == b':' &&       // line 38: must be followed by ':'
                   (j + 1 >= len || bytes[j + 1] != b':') { // not '::'
                    let key = line[key_start..key_end].to_string();
                    if seen.contains(&key) {
                        // emit diagnostic                // line 41-49
                    } else {
                        seen.insert(key);                // line 51
                    }
                }
                continue;
            }
            j += 1;
        }
    }
    diags
}
```

**Key structural properties of the current implementation:**

1. It calls `check_source` (line-level scanning), not `check_node` (AST traversal). The `Rule` trait provides both paths; this rule chose the wrong one.
2. The `seen: HashSet<String>` is created fresh **per line** (line 25), not per hash literal.
3. It only recognises the `word:` new-style symbol key syntax. The pattern match at line 31 only enters the key-detection branch when `b.is_ascii_alphabetic() || b == b'_'`.
4. It has no tracking of hash open/close brackets `{` / `}`, array brackets `[` / `]`, or nesting depth of any kind.
5. The key comparison is purely string equality on the identifier text extracted from the source.

---

## Bug 1: String Key Duplicates Not Detected

### Current Code Logic

At line 31, the scanner only enters the key-detection branch when the current byte is ASCII-alphabetic or `_`. A string literal key like `'Mention'` starts with `'` (byte value 39), which is neither alphabetic nor underscore. Therefore the branch is never entered for `'Mention' =>` keys.

The old-style rocket-syntax (`=>`) is not handled at all. The comment at lines 21-22 reads:

```rust
// Extract all `word:` patterns from the line (new hash syntax)
// Also detect `"word" =>` or `:word =>` patterns
```

The comment acknowledges that `"word" =>` and `:word =>` detection was intended but it was never implemented. There is no code path that:
- Skips past a quote character `'` or `"`
- Reads the string content until the closing quote
- Then looks ahead for ` =>`

### Root Cause

The scanner at line 31 gates key detection entirely on `b.is_ascii_alphabetic() || b == b'_'`. String literal keys start with `'` or `"`. These bytes do not satisfy that condition. The `=>` operator is never examined. The result is that old-style hash syntax keys of any type — string, symbol-with-colon (`:symbol =>`), or integer — are completely invisible to the rule.

### Evidence

- Line 31: `if b.is_ascii_alphabetic() || b == b'_'` — the only entry point for key detection.
- No branch in the `while j < len` loop handles `b == b'\''` or `b == b'"'` or `b == b':'` (for `:symbol =>` style).
- Lines 21-22: the comment is aspirational, not implemented.

### Fixture Gap

The fixture at `rubric-rules/tests/fixtures/lint/duplicate_hash_key/offending.rb` contains only:

```ruby
h = {a: 1, b: 2, a: 3}
```

This is a single-line hash with new-style symbol keys. There is no test case for string keys or rocket-syntax keys, so the bug has never been caught by the test suite.

### Proposed Fix Approach

The rule must be rewritten to use AST traversal via `check_node` on `HashNode`. For each `HashNode`, iterate its `AssocNode` children. Each `AssocNode` has a `key` child which is a typed AST node (`StringNode`, `SymbolNode`, `IntegerNode`, etc.). Extract the key's value by type, normalise it to a canonical string (e.g. the string content for `StringNode`, the symbol name for `SymbolNode`), and track seen keys within the scope of that single `HashNode`.

---

## Bug 2: False Positives on Nested Sub-hash Keys

### Current Code Logic

The `seen` set is initialised per-line (line 25 in `duplicate_hash_key.rs`). This means the rule compares all `word:` patterns found on the same source line against each other. In a multi-line hash this is partially contained by line boundaries — but the behaviour is actually worse than cross-line confusion.

The critical flaw is that `seen` is **per-line**, not **per-hash**. For a hash that spans multiple lines, each line is scanned independently. But for the `TARGET_STATUS_INCLUDES_BY_TYPE` structure cited in the bug report, the problematic lines look like:

```ruby
TARGET_STATUS_INCLUDES_BY_TYPE = {
  status: :status,              # outer key: 'status'
  reblog: [status: :reblog],    # 'reblog' is outer key; 'status' is inner key in an array sub-hash
  mention: [mention: :status],  # 'mention' is outer key; 'mention' is inner key in an array sub-hash
  ...
}
```

On the line `  reblog: [status: :reblog],`, the scanner sees two `word:` patterns: `reblog` and `status`. It does not know that `reblog` is in the outer hash and `status` is inside `[...]`. Both are added to `seen` for that line. No immediate false positive on this line alone.

However, on a line that has the structure `  mention: [mention: :status],`, the scanner sees `mention` twice — once as the outer hash key and once as the inner array sub-hash key. Both match the `word:` pattern. The first occurrence is inserted into `seen`. When the second occurrence (the inner key) is scanned, `seen.contains("mention")` is true, so a violation is emitted. This is a false positive.

### Root Cause

The scanner has no awareness of bracket nesting. The `[` and `]` characters are never examined. The `{` and `}` characters are never examined. Any `word:` pattern on a line is treated as a hash key at whatever scope it textually appears, without regard for whether it is inside an array literal `[...]`, a nested hash `{...}`, a method argument list, or any other structure.

The line `  mention: [mention: :status],` contains two syntactically distinct `mention:` tokens:
- `mention:` at column 2 — the key of `TARGET_STATUS_INCLUDES_BY_TYPE`
- `mention:` at column 12 (inside `[...]`) — the key of an inner implicit hash inside the array

The scanner sees them both as bare `word:` patterns and flags the second as a duplicate.

Similarly, consider a line like `  reblog: [status: :reblog],` where `status` on a different earlier or later line is `status: :status`. If these appear on separate lines, the per-line `seen` set means they do not conflict. But the seven reported violations suggest there are lines where the outer and inner keys coincide on the same source line.

### The Seven Violations

With a structure like:

```ruby
TARGET_STATUS_INCLUDES_BY_TYPE = {
  status: :status,
  reblog: [status: :reblog],
  mention: [mention: :status],
  favourite: [favourite: :status],
  reblog_of_owned: [reblog: :reblog],
  mention_of_owned: [mention: :status],
  favourite_of_owned: [favourite: :status],
  ...
}
```

Lines of the form `  mention: [mention: :status],` have `mention:` appearing twice on the same line. The per-line `seen` set will flag the second as a duplicate. Each such line produces one violation, explaining the count of 7.

### Evidence

- Line 25: `let mut seen: HashSet<String> = HashSet::new();` — reset per line, not per hash scope.
- Line 31: `if b.is_ascii_alphabetic() || b == b'_'` — no bracket tracking precedes this; it fires for all `word:` patterns regardless of what brackets surround them.
- There is no variable tracking nesting depth, no stack of open delimiters, and no mechanism to distinguish hash context from array context.

### Proposed Fix Approach

Using AST-based traversal on `HashNode` directly eliminates this bug. The prism AST represents `[status: :reblog]` as an `ArrayNode` containing a `KeywordHashNode`. The `KeywordHashNode` is a separate AST node from the parent `HashNode`. When the rule visits the parent `HashNode`, it only iterates the direct children of that node's `elements` list — it never descends into child arrays or nested hashes unless it explicitly recurses into them. Each `HashNode` / `KeywordHashNode` is checked in isolation.

---

## Why RuboCop Gets This Right

RuboCop's `Lint/DuplicateHashKey` is implemented in `lib/rubocop/cop/lint/duplicate_hash_key.rb` and operates on the Parser gem's AST. Its approach:

1. **Node-level visit:** It registers `on_hash` which is called for each `s(:hash, ...)` AST node. Each `on_hash` call operates on exactly one hash literal — its own isolated scope.

2. **Direct children only:** Inside `on_hash`, it iterates `node.pairs` — the direct key-value pairs of that specific hash node. Nested hashes inside array values are separate `s(:hash, ...)` nodes visited by separate `on_hash` calls. They are never mixed with the parent's pair list.

3. **Key value extraction by type:** Each pair has a key node. RuboCop extracts the key's value using `pair.key.value` or `pair.key.sym_type?` / `pair.key.str_type?`, which gives the actual Ruby string or symbol value rather than source text. This correctly handles `'Mention'` and `:Mention` as separate key types (one is a string, one is a symbol) or two `'Mention'` string literals as equal.

4. **No line-level text scanning:** RuboCop never reads raw source bytes for this check. The AST provides typed, scope-bounded, syntactically accurate information.

Rubric's implementation at `rubric-rules/src/lint/duplicate_hash_key.rs` is doing the equivalent of a regex-on-source approach where RuboCop does a typed AST walk. The `Rule` trait in `rubric-core/src/rule.rs` (lines 22-35) explicitly supports `check_node` for AST-level rules and lists `node_kinds()` as the opt-in mechanism. The prism AST exposes `HashNode` and `KeywordHashNode` (confirmed in `rubric-core/src/walker.rs` lines 79 and 104). The infrastructure for a correct fix exists; it was not used.

---

## Code-Level Root Cause Analysis: Line References

| File | Line(s) | Issue |
|------|---------|-------|
| `rubric-rules/src/lint/duplicate_hash_key.rs` | 11 | Uses `check_source` (text scan) not `check_node` (AST); wrong entry point for a semantic rule |
| `rubric-rules/src/lint/duplicate_hash_key.rs` | 25 | `seen` is scoped per-line, not per-hash — breaks multi-line hashes and does not isolate sub-hashes |
| `rubric-rules/src/lint/duplicate_hash_key.rs` | 31 | Key detection gated on `is_ascii_alphabetic() || b == b'_'`; string literal keys starting with `'`/`"` are invisible |
| `rubric-rules/src/lint/duplicate_hash_key.rs` | 38 | Only `word:` pattern (new-style symbol key) detected; `word =>`, `'str' =>`, `:sym =>` patterns are absent |
| `rubric-rules/src/lint/duplicate_hash_key.rs` | 21-22 | Comment claims `"word" =>` / `:word =>` detection is present; it is not |
| `rubric-rules/src/lint/duplicate_hash_key.rs` | 28-57 | Entire inner loop has no tracking of `[`, `]`, `{`, `}` bracket depth; all `word:` tokens on a line are treated as same-scope hash keys |
| `rubric-rules/tests/fixtures/lint/duplicate_hash_key/offending.rb` | 1 | Fixture only covers the one case the implementation handles: single-line new-style symbol key hash |
| `rubric-rules/tests/duplicate_hash_key_test.rs` | 11-16 | Test only asserts `!diags.is_empty()` — does not verify count or specific keys; would not catch false positives |
| `rubric-core/src/rule.rs` | 22-35 | `node_kinds()` + `check_node()` API exists and is the correct mechanism; unused by this rule |
| `rubric-core/src/walker.rs` | 79, 104 | `HashNode` and `KeywordHashNode` are registered in the walker's kind map; available for use |

---

## Correct Fix Approach Summary

The rule must be rewritten from a `check_source` text scanner to a `check_node` AST visitor. The correct approach is:

1. Implement `node_kinds()` returning `&["HashNode", "KeywordHashNode"]`.
2. In `check_node`, for each hash node, iterate the direct `elements` (AssocNodes).
3. For each `AssocNode`, inspect the key child node's type:
   - `SymbolNode` — extract the symbol name (bytes between the `:` and end of token)
   - `StringNode` — extract the string value (bytes between quotes)
   - `IntegerNode` — extract the integer value
   - Other types (dynamic keys, interpolated strings) — skip; cannot safely deduplicate
4. Build a canonical key representation that includes the key type (so `"foo"` string and `:foo` symbol are not conflated as both being "foo").
5. Maintain a `seen` map scoped to the current `HashNode` call only; it is not shared with any other node visit.
6. Emit a diagnostic for any key whose canonical form has already been seen within the same hash node.

This approach exactly mirrors RuboCop's behaviour and is sound because the prism AST guarantees that each `HashNode` represents exactly one hash literal at exactly one scope level.
