# Research: `Layout/SpaceAroundOperators` False Positives

**Date:** 2026-03-02
**File under analysis:** `rubric-rules/src/layout/space_around_operators.rs`
**Related files read:**
- `rubric-core/src/context.rs`
- `rubric-core/src/rule.rs`
- `rubric-core/src/walker.rs`
- `rubric-rules/tests/space_around_operators_test.rs`
- `rubric-rules/tests/fixtures/layout/space_around_operators/offending.rb`
- `rubric-rules/tests/fixtures/layout/space_around_operators/corrected.rb`

---

## 1. Summary of the Detection Approach

`SpaceAroundOperators` implements `check_source`, not `check_node`. This means it does **not** use the AST at all. It is a byte-level, line-by-line scanner.

The algorithm (lines 10–143 of `space_around_operators.rs`):

1. Iterates over every line in the source file.
2. Walks each line byte by byte, maintaining one piece of state: `in_string: Option<u8>`, which tracks whether the scanner is inside a `"` or `'` string literal.
3. When it encounters a two-byte sequence that matches a set of recognised operators, it checks that the byte before and the byte after are whitespace. If not, it emits a diagnostic.
4. For single-byte operators (`=`, `+`, `-`, `*`, `/`), a similar whitespace check is performed with a small set of "unary" heuristics to skip some false positives.

**There is no regex literal tracking, no heredoc tracking, no AST context, no compound-assignment awareness beyond byte matching, and no parameter-position awareness.**

---

## 2. The Operator Tables

### Two-character operators (lines 36–38)

```rust
if two == b"==" || two == b"!=" || two == b"<=" || two == b">="
    || two == b"&&" || two == b"||" || two == b"+=" || two == b"-="
    || two == b"*=" || two == b"/=" || two == b"**"
```

These are **checked for surrounding spaces**, not skipped. The variable comment on line 35 says "Skip compound operators" but the code body checks spacing on them — every entry in this list generates a diagnostic if not surrounded by spaces. Only `->` (lines 60–63) is actually skipped (no diagnostic produced).

### Single-character operators (lines 68–136)

`=`, `+`, `-`, `*`, `/` are checked. `<` and `>` are mentioned in the comment on line 66 but are NOT present in the `match` arm — they are silently ignored.

---

## 3. False Positive Categories — Root Cause Analysis

### 3.1 `||=` Memoization Operator

**Example:** `@foo ||= some_value`

**What happens in the scanner:**

The scanner walks to position of `|`. It reads a two-byte window `||` and enters the two-char operator branch. It checks whether `||` has spaces on both sides.

In `@foo ||= some_value`, the bytes around `||` are:
- before `||`: `' '` (space) — OK
- after `||`: `'='` — NOT a space or tab

So `next_ok` is false and a diagnostic fires: "Operator `||` should be surrounded by spaces."

The scanner then advances `j` by 2, past the `||`. On the next iteration it lands on `=`. The `=` handler checks `prev` which is now `|`. Since `|` is not in the prev-skip list (`!`, `<`, `>`, `=`), the `=` is also evaluated for spacing. Before it: `|` (no space) — another diagnostic.

**Root cause:** `||=` is a three-character compound assignment token. The scanner has no lookahead for the third character when it matches `||`. It treats `||` as a standalone binary operator and then processes `=` separately. There is no `||=` entry in any skip or exclusion list.

The same logic applies to `&&=` (though not tested here), `**=`, and any other three-character compound assignment.

### 3.2 `**` Double-Splat / Keyword Splat

**Example:** `def foo(**opts)` or `send(method, **args)`

**What happens in the scanner:**

At `**`, the two-char branch matches `b"**"` (line 38). The scanner checks:
- before `**`: `(` or `,` followed by a space — these are not `' '` or `'\t'`
- after `**`: `o` (start of the variable name) — not a space

So `prev_ok` or `next_ok` is false and a diagnostic fires.

**Root cause:** The scanner has no way to determine whether `**` is the exponentiation operator (`x ** 2`) or the keyword splat prefix (`**opts`). In Ruby, `**` as a splat prefix is always directly adjacent to the variable name with no space — that is the canonical, correct style. The scanner cannot tell the difference because it has no AST and no structural context about whether it is inside a parameter list or an argument list.

In `def account_number(digits: 10)` (Faker example), any `**` in a nearby method definition for default keyword args would be treated as binary exponentiation.

### 3.3 Regex Character Class Operators — `/[a-z0-9_]+.../`

**Example:** `/[a-z0-9_]+([.-]+[a-z0-9_]+)*/`

**What happens in the scanner:**

The `in_string` state (lines 18, 27) only tracks `"` and `'` delimiters. There is no `in_regex` state. When the scanner encounters `/`, it does not enter any regex literal mode.

Inside `/[a-z0-9_]+([.-]+[a-z0-9_]+)*/`, the scanner will:
- See `-` inside `[a-z0-9_]` — the character class range operator, which has `0` before it and `9` after it, no spaces. The unary heuristic (line 110) checks if `prev` is `(`, `[`, `,`, space, `=`, `+`, `-`, `*`, `/`. Here `prev` is `0` (digit), so the unary guard does not fire. A diagnostic is emitted.
- See `+` after `]` — `+` is a quantifier. `prev` is `]`, not in the unary skip list. `next` is `(`, not a space. Diagnostic fires.
- See `.` — not handled (`.` is not in the operator match arms), so this one is safe.
- See `*` at the end of `([.-]+[a-z0-9_]+)*` — `prev` is `)`, not in the unary skip list. `next` is `/` (closing delimiter), not a space. Diagnostic fires.

**Root cause:** No regex literal detection. The scanner has only single-quote and double-quote string tracking. Slash-delimited regex literals, `%r{...}` literals, and their contents are treated as plain code.

### 3.4 `%r{...}` Regex Literals

**Example:** `%r{(?<![=/[:word:]])@...}`

**What happens in the scanner:**

`%r{...}` is not recognised at all. The scanner sees `%` (not in any operator arm, so skipped), then `r`, `{`, and then processes the contents character by character as if they were regular Ruby code. Any `-`, `+`, `*`, `=`, `<`, `>` inside the pattern body are checked as operators.

Notably, inside `(?<![=/[:word:]])`, the `=` and `/` and `[` and `]` appear. The `=` handler (line 69) would check the `=` here. `prev` would be `<` which is not in the skip list, so a diagnostic fires.

**Root cause:** Same as 3.3 — no `%r{...}` literal detection.

### 3.5 Alignment `=` in Hash and Variable Groups

**Example:**
```ruby
@text    = value1
@options = value2
```

**What happens in the scanner:**

In `@text    = value1`, the `=` is at some position with multiple spaces before it. The `=` handler (line 85) checks:
```rust
let prev_ok = j == 0 || prev == b' ' || prev == b'\t';
let next_ok = next == b' ' || next == b'\t' || next == 0;
```

`prev` is `' '` (space) and `next` is `' '` (space), so both are OK and no diagnostic fires.

**Wait — this might not be a false positive caused by the spacing check itself.** Let me re-examine the alignment case more carefully.

In aligned groups the `=` does have a space on both sides. The bug report says these are flagged. The likely cause is that `in_string` state leaks across lines. Examining the scanner: `in_string` is declared at line 18 and is **not reset between lines** — it persists across the outer `for` loop over lines. If a `"` appears without a closing `"` on the same line (e.g., a string value that is continued, or a heredoc, or a line ending inside an interpolation), the `in_string` state will carry forward to subsequent lines, suppressing diagnostics on those lines. Conversely, if `in_string` is set from a previous line, an `=` in what appears to be an alignment group might be skipped (not flagged), or the `#` comment guard (line 28) might fire early due to state confusion.

However, looking at the code again: `in_string` is initialised to `None` inside the `for (i, line)` loop body (line 18). Each line starts fresh. So cross-line string leakage does not apply.

The alignment `=` diagnostic is more likely triggered by something subtler. If `@text    = value1` has multiple spaces before `=`, then `prev` is `' '`. That passes. But if there is a tab (`\t`) used for alignment and the scanner byte-by-byte hits the `=`, it still passes since `\t` is allowed.

**Re-evaluation:** The alignment case in the bug report (48 firings for 5 files with only 1 injected violation) may not be the `=` firing — it may be the `*` or `-` in variable names or values on those lines. Or the `=` may not be firing at all and was misattributed. The core alignment scenario (space on both sides of `=`) should pass the current check. More investigation with real Mastodon file content would be needed.

---

## 4. Why the Test Suite Does Not Catch These

The fixture files are trivially simple:

`offending.rb`:
```ruby
x=1
y=x+2
z = x==y
```

`corrected.rb`:
```ruby
x = 1
y = x + 2
z = x == y
```

There are no fixtures for:
- `||=` memoization
- `**` splat parameters
- Regex literals of any form
- Hash alignment
- Multi-character compound assignments (`&&=`, `**=`, `||=`)
- Unary operators in complex positions

The test `no_violation_with_spaces_around_operators` only asserts `diags.is_empty()` on the corrected fixture. It does not test any of the false-positive patterns.

---

## 5. How RuboCop Avoids These False Positives

RuboCop's `Layout/SpaceAroundOperators` cop operates on the AST produced by Parser gem (or RuboCop AST). It never scans token text for `||` or `**` raw characters. Instead:

- **`||=`** produces an `or_asgn` or `lvasgn`/`ivasgn` node with `||=` as the operator. The cop visits the assignment node directly and knows the full operator token, not sub-sequences of it.
- **`**` splat** produces `kwsplat` nodes in argument lists and `kwoptarg`/`kwrestarg` nodes in parameter lists. These node types are explicitly excluded from the operator spacing check — the cop only fires on binary operator nodes (`send` with an operator method name such as `:**`, `:+`, etc.).
- **Regex literals** produce `regexp` AST nodes. The cop does not descend into the literal content of regexp nodes. All metacharacters inside a regexp are invisible to the operator spacing check.
- **Compound assignments** (`+=`, `-=`, `||=`, `&&=`, `**=`) all produce distinct AST node types (`op_asgn`, `or_asgn`, `and_asgn`). The cop matches on the node type, not the raw characters. An `or_asgn` node with `||=` will never be matched by the binary `||` check.
- **String contents** — string node contents are never scanned for operators.

In short: RuboCop's approach is that the grammar has already decomposed source text into semantically typed nodes before any cop sees it. The cop only visits operator-bearing nodes that are unambiguously binary operations in context.

---

## 6. Summary Table of False Positive Root Causes

| False Positive | Scanner Behaviour | Root Cause |
|---|---|---|
| `@foo \|\|= val` flagged for `\|\|` | `\|\|` matched at line 37, spacing check fails on `=` to the right | No lookahead for three-character `\|\|=` token; treats `\|\|` as standalone binary operator |
| `**opts` / `**args` flagged for `**` | `**` matched at line 38, `prev` is `(` or `,`+space, `next` is letter | No positional/structural context; cannot distinguish splat prefix from exponentiation |
| `/[a-z]-[z]/` flagged for `-` | `-` encountered inside regex body, `prev` is alphanumeric | No regex literal tracking; only `"` and `'` strings are tracked as exclusion zones |
| `%r{...}` content flagged | `%r` not recognised; content scanned as code | No `%r` literal detection at all |
| `&&=` (implied, same pattern as `\|\|=`) | `&&` matched, spacing check fails on `=` | Same as `\|\|=` case |
| `**=` (exponent-assign) | `**` matched, `next` is `=`, fails spacing | No three-char lookahead |

---

## 7. What Exclusion Rules Need to Be Added

### 7.1 Three-character compound assignments

Before matching any two-character operator, the scanner must look at the third byte. If the two-char sequence is `||`, `&&`, or `**`, and the byte at `j+2` is `=`, the three characters form a compound assignment token and must be consumed as a unit (advance `j += 3`) with no diagnostic.

Same applies to `**=` (not currently in the two-char list but could appear).

### 7.2 Double-splat / keyword splat context

When `**` is encountered, the scanner must determine whether it is:
- A binary operator: appears between two value-producing expressions, with whitespace on both sides in idiomatic Ruby.
- A splat prefix: appears immediately after `(`, after `,` (possibly with space), or after `[` — i.e., it directly prefixes a variable name with no space between `**` and the name.

Heuristic: if `prev` (ignoring whitespace) is `(`, `,`, `[`, or the byte sequence `**` is at the start of a line (after optional whitespace), treat it as splat prefix and skip without diagnostic.

### 7.3 Regex literal tracking

The scanner must track `/` delimiters for regex literals. This is complicated because `/` is also the division operator — disambiguation requires knowing whether the `/` appears after a value (division) or after an operator/keyword/open paren (regex). A minimal heuristic: if the previous non-whitespace byte is `(`, `,`, `=`, `!`, `|`, `&`, a keyword character, or start of line, treat the `/` as opening a regex literal. Scan forward to the closing unescaped `/` (skipping character classes `[...]` and escaped chars) and advance past the entire literal.

### 7.4 `%r{...}` and other `%`-literal forms

The scanner must recognise `%r` followed by a delimiter character and track the matching closing delimiter, treating the entire span as an opaque literal that is not scanned for operators.

### 7.5 General literal-aware scanning architecture

The real fix is to extract a `LiteralScanner` pre-pass that marks byte ranges in each line as one of: `Code`, `StringLiteral`, `RegexLiteral`, `Comment`. The operator check then only operates on `Code` ranges. This is how any correct implementation must work.

---

## 8. Architectural Observation

The `Rule` trait (in `rubric-core/src/rule.rs`) already provides `check_node` with AST-level dispatch via ruby-prism. The walker (in `rubric-core/src/walker.rs`) shows the infrastructure exists: ruby-prism is already a dependency and parses Ruby into a full AST with distinct node types for every operator context.

`SpaceAroundOperators` was implemented as a `check_source` rule (byte scanner) when it should be a `check_node` rule visiting binary operator nodes. The relevant ruby-prism node types are:

- `CallNode` — binary operator calls (e.g., `x + y`, `x ** y`, `x / y`)
- `LocalVariableOrWriteNode`, `InstanceVariableOrWriteNode`, etc. — `||=` forms
- `LocalVariableOperatorWriteNode`, `InstanceVariableOperatorWriteNode`, etc. — `+=`, `-=`, `*=`, etc.
- `KeywordRestParameterNode`, `AssocSplatNode` — `**` in parameter/argument position (these would be excluded, not checked)
- `RegularExpressionNode`, `InterpolatedRegularExpressionNode` — regex literals (excluded from scanning)

The walker in `rubric-core/src/walker.rs` (lines 185–203) already dispatches to `check_node` based on `node_kinds()`. Migrating `SpaceAroundOperators` to this path would eliminate every known false positive category by construction.

---

## 9. File Locations for Reference

| File | Path |
|---|---|
| Rule implementation | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-rules/src/layout/space_around_operators.rs` |
| Test file | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-rules/tests/space_around_operators_test.rs` |
| Offending fixture | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-rules/tests/fixtures/layout/space_around_operators/offending.rb` |
| Corrected fixture | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-rules/tests/fixtures/layout/space_around_operators/corrected.rb` |
| Rule trait | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-core/src/rule.rs` |
| AST walker | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-core/src/walker.rs` |
| LintContext | `/Users/md.tihami/Desktop/Learn/Projects/Personal/Rusty/rubric/rubric-core/src/context.rs` |
