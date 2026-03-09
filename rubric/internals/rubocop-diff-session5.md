# Rubocop vs Rubric Diff — Session 5 (2026-03-04)

Comparison on 585 mastodon files (the subset flagged by rubric's top candidate rules).
Both tools run with default/empty config (no project .rubocop.yml, no rubric.toml).
Rubocop version: 1.85.0

---

## Confirmed FPs (rubric fires, rubocop enabled=true but fires 0 on same files)

| Rule | Rubocop | Rubric | Excess | Status |
|------|---------|--------|--------|--------|
| Layout/ExtraSpacing | 0 | 60 | +60 | ✗ FP — confirmed same-file test |
| Layout/BlockAlignment | 0 | 52 | +52 | ✗ FP — confirmed same-file test |
| Lint/ParenthesesAsGroupedExpression | 0 | 47 | +47 | ✗ FP — confirmed same-file test |
| Layout/MultilineOperationIndentation | 0 | 41 | +41 | ✗ FP |
| Layout/SpaceAroundOperators | 3 | 43 | +40 | ✗ FP (40 excess) |
| Style/HashSyntax | 0 | 36 | +36 | ✗ FP |
| Layout/EndAlignment | 0 | 35 | +35 | ✗ FP |
| Lint/UselessAssignment | 2 | 34 | +32 | ✗ FP (32 excess) |
| Layout/EmptyLinesAroundClassBody | 0 | 31 | +31 | ✗ FP |
| Layout/EmptyLinesAroundMethodBody | 1 | 30 | +29 | ✗ FP (29 excess) |
| Style/Not | 0 | 25 | +25 | ✗ FP |
| Lint/UnusedMethodArgument | 1 | 26 | +25 | ✗ FP (25 excess) |
| Lint/ConstantDefinitionInBlock | 0 | 25 | +25 | ✗ FP |
| Style/BlockDelimiters | 0 | 19 | +19 | ✗ FP |
| Lint/NonLocalExitFromIterator | 0 | 16 | +16 | ✗ FP |
| Layout/FirstArgumentIndentation | 0 | 16 | +16 | ✗ FP |
| Layout/RescueEnsureAlignment | 0 | 14 | +14 | ✗ FP |
| Style/EmptyMethod | 0 | 13 | +13 | ✗ FP |
| Style/Lambda | 0 | 12 | +12 | ✗ FP |
| Layout/SpaceInsideParens | 0 | 10 | +10 | ✗ FP |
| Layout/SpaceInsideHashLiteralBraces | 0 | 10 | +10 | ✗ FP |
| Layout/MultilineHashBraceLayout | 0 | 9 | +9 | ✗ FP |
| Layout/ElseAlignment | 0 | 8 | +8 | ✗ FP |
| Lint/UnusedBlockArgument | 0 | 7 | +7 | ✗ FP |
| Lint/UnderscorePrefixedVariableName | 0 | 7 | +7 | ✗ FP |
| Lint/EmptyExpression | 0 | 7 | +7 | ✗ FP |
| Lint/AmbiguousOperator | 0 | 7 | +7 | ✗ FP |
| Layout/DefEndAlignment | 0 | 7 | +7 | ✗ FP |
| Lint/AssignmentInCondition | 0 | 6 | +6 | ✗ FP |

## Rules disabled in rubocop by default (rubric fires anyway)

| Rule | Rubocop (disabled) | Rubric | Note |
|------|-------------------|--------|------|
| Style/Send | 0 (Enabled: false) | 58 | Rubic shouldn't fire this by default |
| Style/ReturnNil | 0 (Enabled: false) | 11 | Rubic shouldn't fire this by default |

## LineLength discrepancy

| Rule | Rubocop | Rubric | Excess |
|------|---------|--------|--------|
| Layout/LineLength | 852 | 1074 | +222 |

Needs investigation — both tools use max=120 by default but rubric fires 222 more times.
Likely cause: rubric counts lines inside heredocs, rubocop does not (heredoc body lines are string content).

## Close / in sync (good)

- Style/StringLiterals: 1950 rubocop, 1952 rubric (+2) ✓
- Style/TrailingCommaInArrayLiteral: 208 rubocop, 200 rubric (-8) ✓ close
- Style/PercentLiteralDelimiters: 465 rubocop, 502 rubric (+37) — investigate
- Style/SymbolArray: 451 rubocop, 467 rubric (+16) — investigate
- Style/ClassAndModuleChildren: 149 rubocop, 158 rubric (+9) — investigate

## Missed detections (rubric fires less than rubocop)

These are rules rubric under-implements — not FPs but gaps:

| Rule | Rubocop | Rubric | Gap |
|------|---------|--------|-----|
| Style/TrailingCommaInHashLiteral | 657 | 0 | -657 — not implemented |
| Metrics/* (all) | 400–650 each | 0 | not implemented |
| Style/WordArray | 201 | 62 | -139 |
| Style/IfUnlessModifier | 135 | 5 | -130 |
| Style/Documentation | 457 | 27 | -430 |
| Layout/FirstHashElementIndentation | 62 | 0 | -62 |
| Layout/MultilineMethodCallIndentation | 54 | 0 | -54 |
| Style/HashAsLastArrayItem | 40 | 0 | not implemented |
| Style/MutableConstant | 20 | 0 | our fix may have over-suppressed |

---

## Priority fix order

**Batch 1 (highest excess, each file is independent):**
1. Layout/ExtraSpacing (60 FPs)
2. Layout/BlockAlignment (52 FPs)
3. Lint/ParenthesesAsGroupedExpression (47 FPs)
4. Layout/MultilineOperationIndentation (41 FPs) — spot-check first
5. Layout/SpaceAroundOperators (40 excess)

**Batch 2:**
6. Style/HashSyntax (36 FPs)
7. Layout/EndAlignment (35 FPs)
8. Lint/UselessAssignment (32 excess)
9. Layout/EmptyLinesAroundClassBody (31 FPs)
10. Layout/EmptyLinesAroundMethodBody (29 excess)

**Batch 3:**
11. Style/Not (25 FPs) + Style/Send (58, disabled cop)
12. Lint/ConstantDefinitionInBlock (25 FPs)
13. Layout/LineLength (+222 excess — heredoc body lines)
14. Style/BlockDelimiters (19 FPs)
15. Style/ReturnNil (11, disabled cop) — just disable in rubric defaults

---

## How confirmed

For ExtraSpacing, BlockAlignment, ParenthesesAsGroupedExpression:
- Ran `rubocop --only Layout/ExtraSpacing <file>` on the exact file rubric flagged
- Rubocop: "no offenses detected" on all three
- Rubric: fires warning on all three
- Conclusion: rubric implementation is more aggressive than rubocop's
