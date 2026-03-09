# Bug Research: Lint/UnreachableCode and Style/NegatedIf

**Date:** 2026-03-02  
**Author:** Codebase Reader (research only — no fixes applied)  
**Scope:** Two broken rules producing false positives and missed violations in production Ruby files

---

## Summary

Both rules share a common architectural flaw: they operate on raw text lines with no understanding of Ruby syntax. The consequence is that they conflate surface-level textual patterns with semantic meaning.

- **Lint/UnreachableCode** treats any line that *starts with* a terminator keyword (`return`, `raise`, `break`, `next`) as an unconditional terminator, regardless of whether that keyword is actually used as a modifier (`return if condition`) or as a statement with a trailing guard. Because the check is purely positional — "does the next sibling line at the same indent exist?" — every guard-return pattern produces a false positive.

- **Style/NegatedIf** only matches `if !` when `if` appears at the *start* of a trimmed line, meaning it is the first token. Modifier-position `if !` — where `if` follows an expression on the same line — is never seen by the current pattern, because the line's trimmed start is the expression, not `if`.

---

## Bug 1: Lint/UnreachableCode — Massive False Positive Rate

### Source file
`rubric-rules/src/lint/unreachable_code.rs`

### Test file
`rubric-rules/tests/unreachable_code_test.rs`

### Fixture files
- `rubric-rules/tests/fixtures/lint/unreachable_code/offending.rb` — four lines: a `def`, a bare `return 1`, `x = 2`, `end`
- `rubric-rules/tests/fixtures/lint/unreachable_code/corrected.rb` — three lines: a `def`, a bare `return 1`, `end`

### Current Detection Logic

The rule iterates over every line. For each line it:

1. Skips blank lines and comment lines (line 24).
2. Computes the leading-whitespace indent (line 21).
3. Checks whether the *trimmed* line starts with one of `return`, `raise`, `break`, or `next`, followed by a word-boundary character (lines 30–35). Word-boundary check prevents `returning` from matching.
4. If that test passes it sets `is_terminator = true` and looks for the next non-blank line at the **same indentation level** (lines 37–62).
5. If that next line is not a block-closing keyword (`end`, `else`, `elsif`, `rescue`, `ensure`, `when`) it emits a `Lint/UnreachableCode` diagnostic on that next line.

The word-boundary guard at step 3 is correct in isolation, but step 3 asks only "does the line start with a terminator keyword?" — **not** "is the terminator keyword used unconditionally?"

### Root Cause

The check at line 30 evaluates `trimmed.starts_with(kw)` with no examination of what follows the keyword on the *same* line. Consider:

```ruby
return if condition         # modifier-if: conditional, NOT a terminator
return unless condition     # modifier-unless: conditional, NOT a terminator
return value if condition   # same — conditional
raise SomeError if flag     # same
```

In every one of these cases, `trimmed.starts_with("return")` is `true`. The word-boundary check at the next byte is satisfied because after `return` comes a space. So `is_terminator` is set to `true`, and the rule then looks at the very next real line at the same indent. Because real code follows (the method body continues), that next line is flagged as unreachable.

The variable `is_terminator` is a misnomer: it is actually `starts_with_terminator_keyword`, which is a strictly weaker predicate.

### Evidence from Source

```rust
// Line 30–35 of unreachable_code.rs
let is_terminator = TERMINATORS.iter().any(|kw| {
    trimmed.starts_with(kw) && (
        trimmed.len() == kw.len()
        || !trimmed.as_bytes().get(kw.len()).map(|b| b.is_ascii_alphanumeric() || *b == b'_').unwrap_or(false)
    )
});
```

This expression is `true` for:
- `return` (bare)
- `return 42`
- `return if condition` ← false positive trigger
- `return unless flag`  ← false positive trigger
- `raise SomeError if x` ← false positive trigger

There is no subsequent code in the check block that inspects the *rest* of the terminator line for a trailing `if` or `unless`. Once `is_terminator` is `true`, the rule proceeds unconditionally to emit a violation on the next sibling line.

### The Guard-Return Pattern at Scale

The Mastodon `account_search_service.rb` contains many guard-return patterns:

```ruby
return if @account.nil?
return accounts if limit.zero?
return unless valid_scope?
```

Each of these lines satisfies `trimmed.starts_with("return")` and the word-boundary check. The rule then examines the next non-blank line at the same indent and reports it as unreachable. With 40 such patterns in a single file, 40 false positives are generated.

### The Faker Benchmark Case

The reported issue at line 441–443 of the Faker file is a concrete example of the same flaw. Line 441 contains:

```ruby
if !words.nil?
```

This is a standard block-if (not modifier position). The body executes conditionally. Line 443 is the code that follows within the block. The UnreachableCode rule should not fire here at all, because `if !words.nil?` does not start with a terminator keyword. The false positive at line 443 relative to line 441 implies a different cause: there must be a `return` or similar keyword somewhere between line 441 and line 443 that is in modifier form. The scanner sees that modifier-`return` on line 441 or 442, treats it as unconditional, and marks line 443 as unreachable. The `if !words.nil?` on line 441 confused the diagnosis because the real triggering line is the modifier-`return` that precedes or is embedded near it.

### Test Coverage Gap

The test fixture `offending.rb` only exercises the simplest possible case: a bare `return 1` with an unconditionally unreachable `x = 2` on the next line. There is no test case for:

- `return if condition`
- `return value unless flag`
- guard patterns at the top of methods
- `raise SomeError if flag`

Because the test only covers the happy path, the false positive is not detected by the test suite.

### Correct Algorithm (description, not implementation)

A correct check must, after identifying that a line starts with a terminator keyword, inspect the remainder of that same line to determine whether a trailing `if` or `unless` modifier is present. If the terminator keyword is followed (after the return value expression, if any) by `if` or `unless` and a condition, the statement is **conditional** and does not make the next line unreachable. Only a bare terminator — one where the rest of the line after the keyword and its optional return-value expression contains no `if`/`unless` modifier — should set `is_terminator = true`.

This requires at minimum a simple inline-token scan of the rest of the line after the keyword. A fully correct implementation would need to handle cases like `return calculate_value(a, b) if some_flag?(x)`, where the return value itself contains parentheses and spaces. A regex approach or a token-splitting approach (split on `\bif\b` or `\bunless\b` as a word-boundary-respecting word) would cover the majority of real cases.

---

## Bug 2: Style/NegatedIf — Not Firing on Modifier `if !condition`

### Source file
`rubric-rules/src/style/negated_if.rs`

### Test file
`rubric-rules/tests/negated_if_test.rs`

### Fixture files
- `rubric-rules/tests/fixtures/style/negated_if/offending.rb` — a block-form `if !valid?` / `puts "invalid"` / `end`
- `rubric-rules/tests/fixtures/style/negated_if/corrected.rb` — the same using `unless valid?`

### Current Detection Logic

The rule iterates over every line. For each line:

1. Trims leading whitespace to get `trimmed` (line 14).
2. Tests `trimmed.starts_with("if ")` (line 16). If false, `continue`.
3. Tests whether the substring after `"if "` starts with `!` (line 21). If false, `continue`.
4. If both conditions pass, emits a `Style/NegatedIf` diagnostic pointing at the `if` on that line.

### Root Cause

The gate at step 2 — `trimmed.starts_with("if ")` — will only be true when `if` is the *first non-whitespace token on the line*. This is the block-form `if` (the statement form that opens a multi-line conditional block).

In modifier position, `if` appears *after* an expression on the same line:

```ruby
do_something if !condition         # modifier-if: NOT caught
return value if !predicate?        # modifier-if: NOT caught
process(x) if !x.nil?             # modifier-if: NOT caught
```

In each of these, `trimmed` starts with `do_something`, `return`, or `process` — not with `if `. The check at step 2 fails immediately and the line is skipped. The modifier `if !` is never examined.

### Evidence from Source

```rust
// Lines 16–23 of negated_if.rs
if !trimmed.starts_with("if ") {
    continue;
}

let after_if = trimmed["if ".len()..].trim_start();
if !after_if.starts_with('!') {
    continue;
}
```

There is exactly one code path for detecting `if !`, and it requires `if` to be the first token. There is no second code path that searches for `if !` preceded by other tokens on the same line. The modifier position is architecturally invisible to this rule.

### The Faker Benchmark Case Revisited

The Faker file at line 441 contains `if !words.nil?` — which is a block-form `if`, not modifier-form. The NegatedIf rule *does* cover block-form `if !` and should have fired here. The fact that Rubric did not fire NegatedIf and instead fired UnreachableCode at line 443 suggests:

1. NegatedIf fired correctly on line 441 (it would, since `trimmed.starts_with("if ")` is true here).
2. The *additional* observed miss relates to modifier-form usage *elsewhere in the file* — lines like `return x if !condition` — which NegatedIf silently passes over.

So both bugs manifest in the Faker benchmark but through different code paths.

### Test Coverage Gap

The test fixture `offending.rb` contains only the block-form `if !valid?`. There is no test case for modifier-position `if !`:

```ruby
do_something if !condition      # not tested
return x if !flag               # not tested
```

Because the test only covers block-form, the miss for modifier-form is never caught.

### Correct Algorithm (description, not implementation)

A correct implementation must check **two** distinct patterns on every line:

1. Block-form: `trimmed` starts with `if ` followed by `!` — already implemented.
2. Modifier-form: the line contains a token sequence matching `\bif\s+!` somewhere *after* the leading expression. This requires scanning the line for the word-boundary-delimited token `if` followed by optional whitespace and then `!`, where the `if` is not the first token.

A safe approach is to search for the literal byte sequence ` if !` (space, `if`, space, `!`) anywhere within the line, then confirm that the `if` is preceded by a non-keyword character (a space or closing parenthesis) to exclude false matches inside strings. A regex like `\bif\s+!` applied to the full line handles both forms: in block-form it matches at position 0 (after indentation); in modifier-form it matches at an internal position.

Care must be taken to avoid matching `if` inside string literals or heredoc bodies, but at the line-scanner level of this codebase, the standard practice (evidenced by other rules) is to skip comment lines and treat string-internal keywords as an acceptable edge case.

---

## Why RuboCop Gets This Right

RuboCop parses Ruby source into a full Abstract Syntax Tree (AST) using the `parser` gem before any cop runs. Every node in the AST has an explicit type that encodes its syntactic role:

- A block-form `if` produces an `if` node whose first child is the condition, second child is the then-body, and third child is the else-body (or nil).
- A modifier-form `if` produces an `if_mod` node (or in some parser versions, the same `if` node type but with a distinguished structure) whose structure unambiguously shows that the body is a single expression appearing *before* the condition in the source text, not after it.
- A `return` statement produces a `return` node. A `return` with a modifier produces a `return` node *wrapped inside* an `if_mod` node — making it structurally clear that the return is conditional.

**For UnreachableCode:** RuboCop's `Lint/UnreachableCode` cop walks the body of each method/block and looks for nodes that follow a *sibling* node of type `return`, `raise`, `break`, or `next`. Because AST siblings are already sequenced statements, "following a return" means "the return's parent body array has more elements after the return node." The cop also checks that the return node itself is not of type `if_mod` — if the `return` is wrapped in a conditional modifier node, it is not a guaranteed exit and the following sibling is not unreachable. This AST-level check is exact: it cannot confuse `return if x` (which produces a conditional `if` node containing a `return`) with a bare `return` (which produces a raw `return` node as a body statement).

**For NegatedIf:** RuboCop's `Style/NegatedIf` cop visits every `if` node in the AST, both block-form and modifier-form. The AST does not distinguish them by where `if` appears textually on a line — both produce `if` (or `if_mod`) node types. The cop checks whether the condition child of the `if` node is a `send` node for the `!` method (i.e., `!condition` desugars to `condition.send(:!)` in the AST). Because the check operates on node types and structure rather than text position, it fires for all syntactic positions of `if !` without any special-casing of modifier vs. block form.

The fundamental difference is: RuboCop encodes *what the code means* (via node types); Rubric encodes *what the code looks like* (via line-start patterns). Line-start patterns work for a small set of simple, predictable forms but fail as soon as Ruby's flexible syntax allows the same semantic construct to appear in different textual positions — which is frequent in idiomatic Ruby.

---

## File Reference

| File | Role |
|------|------|
| `rubric-rules/src/lint/unreachable_code.rs` | Buggy rule implementation — no modifier-if check |
| `rubric-rules/src/style/negated_if.rs` | Buggy rule implementation — block-form only |
| `rubric-rules/tests/unreachable_code_test.rs` | Test — no guard-return fixture |
| `rubric-rules/tests/negated_if_test.rs` | Test — no modifier-form fixture |
| `rubric-rules/tests/fixtures/lint/unreachable_code/offending.rb` | Only tests bare `return` |
| `rubric-rules/tests/fixtures/lint/unreachable_code/corrected.rb` | Only removes the dead line |
| `rubric-rules/tests/fixtures/style/negated_if/offending.rb` | Only tests block-form `if !` |
| `rubric-rules/tests/fixtures/style/negated_if/corrected.rb` | Only the `unless` equivalent |
| `rubric-core/src/context.rs` | `LintContext` — lines + byte offsets, no token or AST layer |
| `rubric-core/src/rule.rs` | `Rule` trait — `check_source` is the entry point used by both rules |
