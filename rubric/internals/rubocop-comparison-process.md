# Rubocop vs Rubric Comparison Process

Goal: find confirmed FPs — violations rubric emits but rubocop does not, on identical input with no config on either side.

---

## Prerequisites

- Rubocop 1.85.0 at `/opt/homebrew/lib/ruby/gems/4.0.0/bin/rubocop`
- Rubric binary at `target/release/rubric-cli` (build with `~/.cargo/bin/cargo build --release --bin rubric-cli`)
- Test projects at `/Users/md.tihami/Desktop/Learn/Projects/Personal/ruby-projects-to-test/`

---

## Step 1 — Pick a sample set

Use mastodon (largest project, most FP candidates). Pick ~30 representative files:

```bash
find /path/to/mastodon/app/models -name "*.rb" | head -30 > /tmp/sample_files.txt
```

Or pick files that triggered the highest-count rules in rubric:

```bash
exec target/release/rubric-cli check /path/to/mastodon 2>&1 \
  | grep "TrailingCommaInArrayLiteral\|WordArray\|ExtraSpacing\|BlockAlignment" \
  | awk -F: '{print $1}' | sort -u | head -30 > /tmp/sample_files.txt
```

---

## Step 2 — Run rubocop (no config)

```bash
RUBOCOP=/opt/homebrew/lib/ruby/gems/4.0.0/bin/rubocop

$RUBOCOP --no-color --disable-pending-cops \
  --format json \
  $(cat /tmp/sample_files.txt | tr '\n' ' ') \
  > /tmp/rubocop_out.json 2>/dev/null
```

Extract per-rule counts from JSON:

```bash
cat /tmp/rubocop_out.json \
  | python3 -c "
import json, sys, collections
data = json.load(sys.stdin)
counts = collections.Counter()
for f in data['files']:
    for o in f['offenses']:
        counts[o['cop_name']] += 1
for cop, n in counts.most_common():
    print(f'{n:4d}  {cop}')
" > /tmp/rubocop_counts.txt

cat /tmp/rubocop_counts.txt
```

---

## Step 3 — Run rubric (no config)

Make sure there is no `rubric.toml` in the sample directory, or run on individual files:

```bash
exec target/release/rubric-cli check /path/to/mastodon 2>&1 \
  | grep -f /tmp/sample_files.txt \
  | grep -oP '\[\w+/\w+\]' \
  | tr -d '[]' \
  | sort | uniq -c | sort -rn \
  > /tmp/rubric_counts.txt

cat /tmp/rubric_counts.txt
```

If you want rubric to also ignore the rubric.toml, temporarily move it:

```bash
mv /path/to/mastodon/rubric.toml /path/to/mastodon/rubric.toml.bak
exec target/release/rubric-cli check /path/to/mastodon ...
mv /path/to/mastodon/rubric.toml.bak /path/to/mastodon/rubric.toml
```

---

## Step 4 — Diff the counts

```bash
python3 - <<'EOF'
import re, collections

def read_counts(path):
    counts = {}
    for line in open(path):
        line = line.strip()
        if not line: continue
        parts = line.split(None, 1)
        if len(parts) == 2:
            counts[parts[1].strip()] = int(parts[0])
    return counts

rubocop = read_counts('/tmp/rubocop_counts.txt')
rubric  = read_counts('/tmp/rubric_counts.txt')

all_rules = sorted(set(rubocop) | set(rubric))
print(f"{'Rule':<45} {'Rubocop':>8} {'Rubric':>8} {'Diff':>8}")
print('-' * 75)
for rule in all_rules:
    rc = rubocop.get(rule, 0)
    rr = rubric.get(rule, 0)
    diff = rr - rc
    marker = '  *** FP candidate' if diff > 0 else ('  --- missed' if diff < 0 else '')
    print(f'{rule:<45} {rc:>8} {rr:>8} {diff:>+8}{marker}')
EOF
```

Rules where `Diff > 0` (rubric > rubocop) are confirmed FP candidates.
Rules where `Diff < 0` (rubric < rubocop) are missed detections.

---

## Step 5 — Spot-check one rule

Pick the highest-diff rule, e.g. `Style/TrailingCommaInArrayLiteral`. Find 5 examples:

```bash
exec target/release/rubric-cli check /path/to/mastodon 2>&1 \
  | grep "TrailingCommaInArrayLiteral" | head -10
```

Open each file at the flagged line. Check if the same line is flagged by rubocop:

```bash
$RUBOCOP --no-color --only Style/TrailingCommaInArrayLiteral \
  /path/to/flagged_file.rb
```

If rubocop does NOT flag it → confirmed FP → fix the rule in rubric.
If rubocop DOES flag it → genuine violation → leave it.

---

## Step 6 — Fix confirmed FPs

For each confirmed FP rule:

1. Read `rubric-rules/src/{category}/{rule_name}.rs`
2. Understand why rubric fires but rubocop doesn't (different heuristic? missing guard?)
3. Fix the rule and add a test case
4. Re-run Step 2–4 to verify the diff shrinks

---

## Current top FP candidates (post session 5)

These rules have the highest remaining counts and are worth checking first:

| Rule | Mastodon | Sidekiq | Devise | Likely reason |
|------|----------|---------|--------|---------------|
| `Style/TrailingCommaInArrayLiteral` | 200 | — | — | rubocop allows trailing comma in multiline with `consistent_comma` style |
| `Style/WordArray` | 62 | 36 | — | rubocop may not flag all `%w` opportunities |
| `Layout/ExtraSpacing` | 60 | 3 | 13 | alignment spacing is often allowed |
| `Layout/BlockAlignment` | 52 | 2 | 8 | complex multi-line block edge cases |
| `Lint/ParenthesesAsGroupedExpression` | 47 | 13 | 1 | intentional grouping patterns |
| `Style/SymbolArray` | — | 5 | 56 | similar to WordArray |
| `Style/PercentLiteralDelimiters` | — | 5 | 40 | project style preference |
| `Layout/SpaceAroundOperators` | 43 | — | 6 | alignment / matrix spacing patterns |

---

## Notes

- The `rubric.toml` in each test project disables rules that are known config differences (not bugs). When comparing defaults, ignore those rules in the diff.
- Rubocop's `Style/TrailingCommaInArrayLiteral` has `EnforcedStyleForMultiline` config — default is `no_comma`. If mastodon's rubocop.yml sets `consistent_comma`, that explains the count.
- Some cops in rubric have no rubocop equivalent (they may be new or renamed). Skip those in the diff.
- Rubocop uses AST; rubric uses line-based heuristics. For complex rules, some divergence is expected and acceptable.
