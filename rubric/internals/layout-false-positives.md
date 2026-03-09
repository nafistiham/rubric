# Layout Rule False Positives — Root Cause Analysis

**Date:** 2026-03-02  
**Scope:** Three layout rules generating significant false positives against Faker and Mastodon corpora, plus a supporting note on IndentationWidth.  
**Status:** Research only — no code changes made.

---

## Overview

All three bugs share a single architectural root cause: every affected rule is implemented as a pure text/byte scan in `check_source`, with no access to the Ruby AST. RuboCop enforces these same rules correctly because it operates on a fully parsed syntax tree, where every `{`, `}`, `(`, `)`, and `#` token carries node-type information. Rubric's `LintContext` (defined in `rubric-core/src/context.rs`) exposes only three things: the raw source string, a `Vec<&str>` of lines, and a `Vec<u32>` of line-start byte offsets. There is no token stream, no bracket-balance stack, no AST node type. Rules that need structural context are forced to guess from surrounding characters — and they guess wrong.

---

## Bug 1: Layout/MultilineMethodCallBraceLayout

**Source file:** `rubric-rules/src/layout/multiline_method_call_brace_layout.rs`  
**Test file:** `rubric-rules/tests/multiline_method_call_brace_layout_test.rs`  
**False positives:** 117 (Faker) + 55 (Mastodon)

### Current Detection Logic

The rule iterates over all lines (starting from line index 1). For each line it applies this filter at line 19:

```rust
if trimmed.ends_with(')') && !trimmed.starts_with(')') && trimmed.len() > 1 {
```

In other words: the line ends with `)` AND is not itself just a `)`. When this matches, the rule scans backwards through preceding lines looking for any line that contains `(` (line 26):

```rust
if prev.contains('(') {
    is_multiline = true;
    break;
}
```

If a `(` is found anywhere in any preceding non-empty line, the rule fires a violation: "Closing `)` of multiline method call should be on its own line."

### Root Cause

The rule hard-codes a single enforcement style — the closing `)` must always be on its own line — without recognising that RuboCop's `MultilineMethodCallBraceLayout` defaults to `EnforcedStyle: symmetrical`. Under the symmetrical style, the position of the closing `)` is legal as long as it mirrors the opening: if the opening `(` is on the same line as the method name, the closing `)` may be on the same line as the last argument. Only when the opening `(` is on its own line (a rare style) must the closing `)` be on its own line too.

The second structural flaw is in the backward scan. The condition `prev.contains('(')` is far too broad. It will match:

- Method definitions: `def foo(a, b)`
- String literals: `puts "hello (world)"`
- Hash key patterns: `format: { with: /\A[a-z0-9_]+\z/i }`
- Any comment containing a parenthesis

There is no bracket-balance tracking. The rule does not verify that the `(` it found is the opening paren of a call whose closing `)` is on the current line. It does not verify the `(` is even unclosed. It simply stops at the first line containing any `(` character and declares a multiline call.

### Evidence from Source

```
Line 19: if trimmed.ends_with(')') && !trimmed.starts_with(')') && trimmed.len() > 1 {
Line 26:     if prev.contains('(') {
Line 27:         is_multiline = true;
```

The test fixture at `tests/fixtures/layout/multiline_method_call_brace_layout/offending.rb` contains only:

```ruby
foo(bar,
    baz)
```

This is the only case the test exercises. The "clean" case in the test (line 17 of the test file) is:

```rust
let src = "foo(\n  bar,\n  baz\n)\n";
```

Here the `(` is on its own line and `)` is on its own line, which happens not to trigger the condition at line 19 (because the last argument line `baz` does not end with `)`). The test suite has no case for the common symmetrical pattern where `foo(arg1,\n     arg2)` is valid.

### How RuboCop Handles This

RuboCop's `MultilineMethodCallBraceLayout` cop operates on `send` AST nodes. It reads:

1. The opening paren location directly from the `CallNode`'s opening-paren token offset.
2. The closing paren location from the `CallNode`'s closing-paren token offset.
3. The line number of the method-name token.

It then applies the configured `EnforcedStyle`:
- `symmetrical` (default): closing paren must be on the same line as the last argument if the opening paren is on the same line as the method name; or on its own line if the opening paren is on its own line.
- `new_line`: closing paren always on its own line.
- `same_line`: closing paren always on the same line as the last argument.

RuboCop never does backward text scanning. It reads the actual call node's structure.

### Proposed Fix Direction

The rule needs access to AST `CallNode` data. It should implement `node_kinds()` returning `&["CallNode"]` and `check_node()` operating on the `CallNode`. The check would compare the line of the opening-paren token against the line of the method-name token, and the line of the closing-paren token against the line of the last argument node. The existing text-scan `check_source` implementation should be removed entirely. The `EnforcedStyle` should default to `symmetrical`.

---

## Bug 2: Layout/LeadingCommentSpace

**Source file:** `rubric-rules/src/layout/leading_comment_space.rs`  
**Test file:** `rubric-rules/tests/leading_comment_space_test.rs`  
**False positives:** 96 (Faker)

### Current Detection Logic

The rule scans every line. When a line's trimmed content starts with `#`, it checks the byte at index 1 (line 38):

```rust
if bytes[1] != b' ' {
```

If the character immediately after `#` is not a space, a violation fires. The rule has three explicit exemptions (lines 27–35):

1. `#!` — shebangs
2. `# encoding:` — encoding magic comments
3. `# frozen_string_literal:` — frozen string literal magic comments

There is no exemption for `##`.

### Root Cause

A `##` comment begins with `#` at index 0 and has `#` (not a space) at index 1. The condition at line 38 evaluates to `true` and a violation fires. Every YARD doc block in Faker starts with `##` on its own line, which is how YARD signals a documentation comment as distinct from a plain implementation comment. This is an explicitly recognised convention — RuboCop's `LeadingCommentSpace` has always exempted `##`.

The fix rule at line 56–65 would then auto-apply, replacing `##` with `# #`, which corrupts YARD documentation.

### Evidence from Source

```
Line 27: if bytes[1] == b'!' { continue; }
Line 33: if after_hash.starts_with(" encoding:") || after_hash.starts_with(" frozen_string_literal:") {
Line 38: if bytes[1] != b' ' {
```

The offending fixture at `tests/fixtures/layout/leading_comment_space/offending.rb` contains only:

```ruby
#this is a comment
x = 1 # this is ok
```

The test exercises only the case `#this`. It has no case for `##`, meaning the false positive against `##` was never caught in testing.

The corrected fixture uses `# this is a comment`. If the auto-fix were applied to a YARD block, `##` would become `# #`, which is not a valid YARD directive and would break documentation generation.

### How RuboCop Handles This

RuboCop's `LeadingCommentSpace` cop, in its source at `lib/rubocop/cop/layout/leading_comment_space.rb`, has an explicit allow-list check:

```ruby
DOUBLE_HASH = /\A##/.freeze

def offense?(comment)
  !comment.text.match?(DOUBLE_HASH) && ...
end
```

The cop skips any comment whose text begins with `##`. RuboCop documentation explicitly states: "This cop checks whether comments have a leading space after the `#` sign (unless it's a special comment like `#!`, `#:nodoc:`, `##` etc.)."

### Proposed Fix Direction

Add a single additional guard before the space check at line 38:

```rust
// Skip double-hash YARD documentation comments `##`
if bytes[1] == b'#' {
    continue;
}
```

This mirrors RuboCop's explicit `##` exemption. The guard belongs between the shebang check (line 27) and the encoding check (line 33), as `##` is the most common exemption case and should short-circuit early.

---

## Bug 3: Layout/SpaceInsideBlockBraces

**Source file:** `rubric-rules/src/layout/space_inside_block_braces.rs`  
**Test file:** `rubric-rules/tests/space_inside_block_braces_test.rs`  
**False positives:** 36 (Mastodon)

### Current Detection Logic

The rule scans every byte of every line looking for `{` and `}` characters. When it finds `{`, it attempts to determine whether it is a block brace or a hash literal by inspecting the previous non-whitespace character (lines 44–58):

```rust
let is_hash_context = matches!(
    prev_nonspace,
    b'=' | b',' | b'(' | b'[' | b'{' | 0
) || pos == line.len() - line.trim_start().len();
```

If `prev_nonspace` is `=`, `,`, `(`, `[`, `{`, or the null byte (nothing before), it is classified as a hash. Otherwise it is treated as a block brace. The `}` check at lines 73–84 has no hash/block discrimination at all — it flags every `}` not preceded by a space, regardless of whether it closes a block or a hash.

### Root Cause

The is-hash heuristic is correct for simple single-line hash literals but fails for the multi-line nested hashes in Mastodon's Elasticsearch query builder. Consider:

```ruby
AccountsIndex.query(
  bool: {
    must: { term: { ... } },
    should: [ ... ],
  }
)
```

On the line `bool: {`, the character before `{` is `:` (a colon, which is not in the allow-list of `=`, `,`, `(`, `[`, `{`). The heuristic therefore classifies this `{` as a block brace. Since the next character is a newline, the rule does not fire on this particular `{`. But on lines like `must: { term: { ... } },` the inner `{ term:` follows a space after `{`, so the outer `{` following `must:` has `:` as its previous non-space character and is again misclassified as a block.

The `}` check is even simpler and makes no distinction at all. Any `}` not preceded by a space is flagged, whether it closes a hash or a block. In the Mastodon hash pattern `{ key: value }` nested forms like `}` at the end of an inline hash get flagged when immediately followed by a comma with no preceding space inside the hash.

The deeper issue is that a `:` preceding `{` is the standard Ruby hash-rocket-free syntax for a keyword argument hash value (`key: { ... }`). The `:` is not in the heuristic's allow-list, so this entire class of keyword argument hash values is misidentified as block braces.

### Evidence from Source

```
Line 56–59:
let is_hash_context = matches!(
    prev_nonspace,
    b'=' | b',' | b'(' | b'[' | b'{' | 0
) || pos == line.len() - line.trim_start().len();
```

The missing character is `b':'`. In Ruby, `key: { ... }` is one of the most common patterns in configuration and DSL code (Rails options hashes, RSpec matchers, Elasticsearch queries). The allow-list was written for positional contexts (`=`, `,`, `(`, `[`) but omitted the symbol-key-value context (`:`).

The `}` check at lines 73–84 has no analogous discriminator at all. It is completely context-free.

The offending test fixture at `tests/fixtures/layout/space_inside_block_braces/offending.rb` contains only:

```ruby
[1, 2].each {|x| puts x}
[1, 2].map {|x| x * 2}
```

Both are genuine block braces. The test suite has no hash literal test cases, so the false-positive class was never exercised.

### How RuboCop Handles This

RuboCop's `SpaceInsideBlockBraces` cop checks only `block` AST nodes. In ruby-parser / RuboCop's AST, a block `{ ... }` is represented as a `block` node, while a hash literal `{ key: value }` is a `hash` node. These are structurally different node types. The cop is registered only for `block` nodes and never visits `hash` nodes at all. There is no need for character-level heuristics because the AST disambiguates them precisely.

In ruby-prism (which Rubric already uses for AST walking — see `rubric-core/src/walker.rs`), the equivalent nodes are `BlockNode` and `HashNode`. The walker already knows how to dispatch to both.

### Proposed Fix Direction

The rule needs to be rewritten to use `check_node` against `BlockNode` exclusively, not `check_source`. It should implement `node_kinds()` returning `&["BlockNode"]` and inspect the opening-brace and closing-brace token byte offsets from the `BlockNode` location, then check adjacent bytes in `ctx.source` to confirm space presence. The current `check_source` implementation should be removed. Adding `b':'` to the `is_hash_context` allow-list in the current implementation would reduce false positives but would not eliminate them, because the `}` check remains completely undiscriminating.

---

## Supporting Note: Layout/IndentationWidth and Alignment-Based Indentation

**Source file:** `rubric-rules/src/layout/indentation_width.rs`

### Detection Logic

The rule checks two conditions for every non-empty line (lines 16–35):

1. If the line starts with a tab character, it fires "Use spaces, not tabs."
2. If the number of leading spaces is greater than zero and is not divisible by 2 (`spaces % 2 != 0`), it fires with the actual count.

### Why It Fires on `delegate` Alignment

Mastodon (and many Rails codebases) uses alignment-based indentation for method arguments and `delegate` chains:

```ruby
delegate :username,
         :email,
         to: :account
```

Here `:email` is indented 9 spaces (to align under `:username`). `9 % 2 != 0`, so the rule fires. Similarly, `validates` calls aligned like:

```ruby
validates :username,
          format: { with: /\A[a-z0-9_]+\z/i },
          length: { maximum: 30 }
```

use 10-space indentation, which is fine (`10 % 2 == 0`), but `delegate` with 9 or 11 spaces triggers the rule. RuboCop's `IndentationWidth` cop only checks lines that are structurally indented by a keyword context (`def`, `class`, `do`/`end`, `if`/`end`, etc.) — it does not check continuation lines that are aligned to a method call's argument position. Rubric's implementation has no concept of continuation lines; it applies the `% 2` check to every line uniformly.

---

## Conclusion: The Common Theme — Pure Text Analysis Without AST Context

All three false-positive bugs, and the IndentationWidth alignment issue, arise from the same architectural gap: `check_source` rules operate on raw text and must guess at syntactic meaning from character sequences. This produces heuristics that work for simple, common cases (the kind found in unit test fixtures) but break on real codebases that use:

- Symmetrical multiline call style (`MultilineMethodCallBraceLayout`)
- YARD documentation conventions (`LeadingCommentSpace`)
- Keyword argument hash values (`SpaceInsideBlockBraces`)
- Alignment-based continuation indentation (`IndentationWidth`)

The `rubric-core` infrastructure already provides the solution. `rubric-core/src/rule.rs` defines `check_node()` and `node_kinds()` as first-class rule hooks. `rubric-core/src/walker.rs` implements a full ruby-prism AST walker (`BlockNode`, `HashNode`, `CallNode`, `DefNode`, all node types are dispatched) and is already integrated into the lint pipeline. Rules that need structural context — anything involving matching paired delimiters, distinguishing block from hash, or understanding call structure — should be node-level rules, not source-level rules. The text-scan approach should be reserved for purely lexical checks (trailing whitespace, line length, end-of-line character) where no structural context is required.

---

## File Reference Map

| File | Role |
|------|------|
| `rubric-core/src/context.rs` | `LintContext` — lines, offsets only, no AST |
| `rubric-core/src/rule.rs` | `Rule` trait — `check_source` and `check_node` hooks |
| `rubric-core/src/walker.rs` | ruby-prism AST walker, dispatches all node kinds |
| `rubric-rules/src/layout/multiline_method_call_brace_layout.rs` | Bug 1 — text scan, wrong style assumption, no bracket balance |
| `rubric-rules/src/layout/leading_comment_space.rs` | Bug 2 — missing `##` exemption |
| `rubric-rules/src/layout/space_inside_block_braces.rs` | Bug 3 — hash/block heuristic missing `:`, `}` check fully undiscriminated |
| `rubric-rules/src/layout/indentation_width.rs` | Supporting note — `% 2` applied to all lines including continuation lines |
| `rubric-rules/tests/multiline_method_call_brace_layout_test.rs` | Tests only `new_line` style, no `symmetrical` coverage |
| `rubric-rules/tests/leading_comment_space_test.rs` | No `##` test case |
| `rubric-rules/tests/space_inside_block_braces_test.rs` | No hash literal test cases |
| `rubric-rules/tests/fixtures/layout/multiline_method_call_brace_layout/offending.rb` | One simple case only |
| `rubric-rules/tests/fixtures/layout/leading_comment_space/offending.rb` | Only `#no-space` case |
| `rubric-rules/tests/fixtures/layout/space_inside_block_braces/offending.rb` | Only genuine block brace cases |
