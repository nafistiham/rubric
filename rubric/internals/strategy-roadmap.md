# Rubric — Strategy & Roadmap Notes

> Internal document covering the FP-reduction plateau, architectural path forward,
> testing methodology, and go-to-market plan. Written after session 8.

---

## 1. Why We Hit a Plateau

After 8 sessions of false-positive reduction, the marginal improvement per session has collapsed:

| Session | Mastodon | Sidekiq | Faker | Devise |
|---------|----------|---------|-------|--------|
| S6 | 975 | 615 | 152 | 536 |
| S7 | 358 | 535 | 113 | 457 |
| S8 | 317 | 522 | 104 | 436 |

Three distinct reasons:

### Reason 1 — Line-scanner architectural ceiling (biggest)

The top remaining violations — `UnusedMethodArgument` (102 sidekiq, 72 mastodon, 21 faker)
— **cannot be correctly fixed with a line scanner**. RuboCop uses a full Ruby AST parser,
so it knows:

- `_arg` prefix = intentionally unused (no flag)
- `**opts` used if `.fetch`/`[]` appears in body (no flag)
- block-forwarded args (`&block` + `yield`) = used (no flag)
- `define_method` / `method_missing` override semantics

To match this we'd need to implement full scope tracking — at which point we've
re-implemented a parser.

### Reason 2 — Missing rubric.toml for sidekiq and devise

Sidekiq's `LineLength(100)` — we don't know if their project sets `Max: 200`.
Their `WordArray(36)` may be disabled. Without a `.rubocop.yml` in the test directory
we can't generate a correct rubric.toml. Those ~136 sidekiq violations are config-noise
with no fixable FP.

### Reason 3 — The remaining violations are real

After 8 sessions of fixes, much of what remains **actually is code that violates the rules**.
Many of the 317 mastodon / 104 faker violations are legitimate style issues that RuboCop
would also flag under the right config.

---

## 2. The Three Paths Forward

### Path 1 — Ship a Real Ruby Parser

Replace the line scanner with Prism (Ruby's official parser, C library, Rust bindings
via `ruby-prism` crate) or `lib-ruby-parser` (pure Rust). All scope-dependent rules
get rewritten as AST visitors.

**Pros**
- Eliminates the architectural ceiling permanently
- `UnusedMethodArgument`, `UselessAssignment`, `RedundantSelf` become correctly implementable
- Every future rule is correct by construction — no more heuristic gymnastics
- Reduces false negatives too (things we currently miss)
- Matches RuboCop semantics exactly
- Rayon parallelism still applies at the file level

**Cons**
- The parser itself is not the hard part — rewriting all 150 rule implementations is
  (3–6 month effort)
- All 150 rules need new test fixtures since `LintContext` API changes
- Transition risk: two parallel implementations or a flag-day cutover
- `lib-ruby-parser` is incomplete — rejects some valid Ruby edge-case syntax
- `ruby-prism` bindings are relatively new — API stability risk
- Parsing adds some latency vs. line reading (still faster than RuboCop overall)

**Bottom line:** Correct long-term answer. Wrong short-term answer — it's a complete
rewrite of the rule engine.

---

### Path 2 — Disable Noisy Default-On Cops

Set `default_enabled() → false` for rules where the line-scanner produces unacceptable
FP rates: `UnusedMethodArgument`, `UnusedBlockArgument`, `UselessAssignment`.

**Pros**
- Zero engineering cost
- Honest — acknowledges the tool's current limitations rather than shipping noise
- Matches real-world linter practice: RuboCop itself ships several cops as `Enabled: false`
- Users who need them can explicitly opt in via rubric.toml
- Stops the "noise erodes trust" problem immediately

**Cons**
- These are genuinely valuable lint rules — disabling them reduces rubric's practical value
- Less strict than the tool we're competing with
- Still produces some correct detections — throwing away true positives with the false ones
- Semantic commitment that's hard to walk back once the AST parser ships
- Doesn't address the underlying problem

**Bottom line:** Correct tactical answer for the transition period, but only if paired
with a clear roadmap to the AST parser.

---

### Path 3 — Accept the Plateau

Ship the tool as-is. ~300–500 violations across the test projects is "good enough"
— users will tune their rubric.toml.

**Pros**
- Zero cost
- Honest signal that the line-scanner approach is near-exhausted
- 317 mastodon violations across 1195 files = 0.26 per file — already quite low
- Real users will have their own rubric.toml from `rubric migrate`, filtering
  project-specific disabled cops

**Cons**
- **FPs erode trust disproportionately.** One false positive on clean code damages
  confidence in all 317 warnings — users stop reading the output
- The plateau is not stable — it gets worse as codebases grow or new rules are added
- `UnusedMethodArgument(102)` in sidekiq means every other warning is suspect
- No path to correct scope-dependent rules without changing the architecture

**Bottom line:** Viable only if declaring the tool "done" at its current capability.
Otherwise it's technical debt accruing interest.

---

## 3. The Recommended Sequence

The three paths are not mutually exclusive:

1. **Now — Path 2:** Disable `UnusedMethodArgument` and `UnusedBlockArgument` by default.
   Mark them "experimental — requires AST parser." Stops the noise immediately.

2. **M7 — Path 1 (hybrid):** Add Prism AST to `LintContext`. Migrate the disabled rules
   to AST-backed implementations. Re-enable them as accurate.

3. **Between — Path 3 by default:** Ship what we have, let users tune rubric.toml,
   focus on formatter and CLI experience.

---

## 4. The Hybrid AST Architecture (M7)

The key insight: **we don't need to rewrite everything.** `LintContext` is the shared
interface. Extend it to carry an optional AST alongside the existing line data:

```rust
// Today
pub struct LintContext<'a> {
    pub source: &'a str,
    pub lines: Vec<&'a str>,
    pub line_start_offsets: Vec<u32>,
    pub path: &'a Path,
}

// After M7
pub struct LintContext<'a> {
    pub source: &'a str,
    pub lines: Vec<&'a str>,
    pub line_start_offsets: Vec<u32>,
    pub path: &'a Path,
    pub ast: Option<PrismParseResult>,  // Some(...) when Prism succeeds
}
```

The `Rule` trait gets an optional second method:

```rust
pub trait Rule {
    fn name(&self) -> &'static str;
    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> { vec![] }
    fn check_ast(&self, ctx: &LintContext) -> Vec<Diagnostic> { vec![] }  // new
}
```

Rules that already work correctly keep `check_source`. New AST-backed rules implement
`check_ast`. The walker calls both. During migration, a rule can have both simultaneously
— validate they agree on real codebases, then delete `check_source`.

### What stays line-scanner forever

Many rules have no reason to ever change:

- `TrailingWhitespace`, `EmptyLines`, `LineLength` — purely lexical, definitionally correct
- `SpaceAroundOperators` (simple cases), `SpaceAfterComma`, `SpaceInsideParens` — token-level
- `LeadingCommentSpace`, `TrailingCommaIn*` — structural

This is exactly what `ruff` does — Python AST for scope-dependent rules, regex for
stylistic rules. The hybrid model is industry-standard for high-performance linters.

### Migration order (by FP impact)

| Priority | Rule | Why AST needed |
|----------|------|----------------|
| 1 | `UnusedMethodArgument` | `_` prefix, block forwarding, method_missing |
| 1 | `UnusedBlockArgument` | Same |
| 2 | `UselessAssignment` | Scope graph — is the variable used in any branch? |
| 3 | `RedundantSelf` | Know what `self.x` refers to at call site |
| 4 | Alignment rules | Optional — line scanner works well enough |

The top two alone would eliminate ~195 violations across the test projects.

### Parser choice: Prism

Use `ruby-prism` (official Ruby parser, C library with Rust bindings):
- Error-recovery capable — produces partial AST for broken files (critical for a linter)
- Maintained by Ruby core team — tracks the language exactly
- `lib-ruby-parser` (pure Rust alternative) often hard-fails on edge-case syntax — avoid

---

## 5. Testing Methodology for Launch

### What's wrong with the current approach

We compare rubric's violation count to rubocop's count and reduce the gap. That measures
**agreement with rubocop**, not **correctness**. Problems:

- **No ground truth.** We never manually reviewed output and classified TP vs FP.
  We assumed rubocop is always right — it isn't.
- **No false-negative measurement.** We fix FPs obsessively but never check how many
  real violations rubric silently misses. A tool with zero FPs but 50% FN rate is useless.
- **5 projects, all tuned against.** Faker and mastodon have had FPs fixed session
  by session. They're the training set. Reporting them at launch is reporting training accuracy.
- **Speed benchmarks are informal.** No reproducible wall-clock comparison against rubocop.

### The 10 projects for launch (all fresh — never touched during development)

| Project | Files (est.) | Why |
|---------|-------------|-----|
| Rails | 5,000+ | The canonical Ruby project |
| Discourse | 4,000+ | Large Rails app, strict rubocop config |
| GitLab CE | 10,000+ | Tests scale claim |
| RSpec | 800 | Test framework — lots of block/proc patterns |
| Grape | 500 | REST API DSL — unusual metaprogramming |
| CarrierWave | 300 | Gem — clean, strict style |
| RuboCop itself | 500 | Meta — if rubric lints rubocop cleanly that's a statement |
| Jekyll | 300 | Static site generator — different idiom set |
| Solidus | 1,500 | E-commerce Rails — real-world complexity |
| Thor | 300 | CLI framework — DSL patterns |

### Methodology per project

1. Run `rubric migrate .rubocop.yml` → generate rubric.toml
2. Run `rubric check` and `rubocop` with matching config on identical file sets
3. Record violations per rule for both tools and wall-clock time for both
4. **Manually sample 20–30 rubric warnings per project and classify TP vs FP**
5. Report precision per rule, not just aggregate counts

### What to measure and report

**Speed** — the strongest claim, easiest to measure:
```
hyperfine "rubric check ./rails" "rubocop ./rails" --warmup 2 --runs 5
```
Report median wall-clock time and the multiple. On a 5,000-file Rails repo, rubric
should be 20–50× faster. That's the headline.

**Precision per rule** — not aggregate counts. For the 20 most common rules:
```
Layout/TrailingWhitespace  — 50/50 correct (100% precision)
Lint/UnusedMethodArgument  — disabled by default (line-scanner limitation)
Layout/EndAlignment        — 18/20 correct (90% precision)
```

**Rule coverage** — a transparent table of which RuboCop cops rubric implements
and which it doesn't.

### What not to do

- Report results on faker/mastodon/sidekiq/devise/puma — training set
- Report aggregate violation counts without methodology
- Claim accuracy without precision/recall numbers per rule
- Compare to rubocop without standardizing config

---

## 6. Go-to-Market

### The one framing that matters everywhere

**"Ruff for Ruby"**

Ruff (Python linter in Rust) went from 0 to 30k GitHub stars in 18 months by being
100× faster than the incumbent. The Ruby community has been explicitly asking
"where is our Ruff?" Rubric is the answer. Use this framing everywhere, always.

---

### Channels ranked by leverage

#### Tier 1 — Do these at launch

**Ruby Weekly Newsletter**
~40,000 subscribers, all Ruby developers. Curated by Peter Cooper. Submit a link at
rubyweekly.com — "Ruby linter written in Rust, 40× faster than RuboCop" is exactly
what they publish. Single highest-leverage action.

**Hacker News — Show HN**
"Show HN: Rubric — Ruby linter in Rust, 40× faster than RuboCop" with solid benchmarks
will hit the front page. That's 50,000 technical readers in 24 hours. This is how Ruff
got its initial breakout. The HN audience is skeptical and technical — benchmarks and
limitations documentation must be bulletproof before posting.

**ruby.social (Mastodon)**
The Ruby community migrated heavily here. Ruby core developers, gem authors, and the
RuboCop maintainer (Bozhidar Batsov — @bbatsov) are active. Tag Batsov — not to
challenge him, but because acknowledgement from the RuboCop creator is signal.

#### Tier 2 — High value, lower effort

**r/ruby** — 35,000 members. Post a technical writeup, not a promotion link.
"I built a Ruby linter in Rust — here are honest benchmarks on 10 real projects"
performs far better than "check out my project."

**dev.to (Ruby tag)** — Write "Why I rewrote a Ruby linter in Rust and what I learned."
Cover technical decisions, include benchmarks, be honest about limitations. This becomes
the link you share everywhere else.

**Twitter/X and Bluesky** — Show something visual. A GIF of `rubric check` completing
on a large Rails repo before RuboCop outputs its first warning is more compelling than
any text benchmark.

#### Tier 3 — Builds over time

**Ruby Podcasts** — Remote Ruby, Ruby Rogues, Rubber Duck Dev. Reach out for a guest
appearance. 3–6 month play but builds credibility with a listening audience.

**RubyConf / RubyKaigi** — Submit a proposal for next year. "Building a Ruby linter in
Rust: performance, correctness, and the road to AST-based analysis" is a strong proposal.
RubyKaigi (Japan) is the most prestigious Ruby conference.

**GitHub Trending** — Can't be controlled directly, but a concentrated burst of stars
from the HN post or Ruby Weekly pushes into GitHub Trending for Ruby. Once there,
organic discovery compounds for days.

---

### What you need before going public

**Must have:**
- `gem install rubric` works in 30 seconds and runs without errors on a real project
- Benchmarks on 5+ projects never tuned against, methodology documented
- README answers in the first paragraph: what it is, how fast, how to install, which cops
- Honest limitations section: "150 of 400+ RuboCop cops; scope-dependent rules
  (UnusedMethodArgument) disabled by default pending AST migration in M7"

**Should have:**
- Comparison table: rubric vs rubocop (speed, cop coverage, config compatibility)
- `rubric migrate` working reliably — "reads your existing .rubocop.yml" is the biggest
  adoption friction reducer
- At least one real project using rubric in their CI

**Nice to have:**
- Short screen recording of the speed difference
- A single-page GitHub Pages site — makes it feel like a real project

---

### Positioning

**Best early use case:** rubric as a fast pre-commit check (catch obvious things in <1s)
while keeping rubocop as a slower CI gate. Zero switching cost. Immediately demonstrates
the speed value. Lead with this use case in all positioning.

**The realistic timeline:** One good HN post → Ruby Weekly pickup → first 500 stars
→ podcast appearances → conference talk → serious adoption. 6–18 months. The first
post starts the clock.

---

*Last updated: session 8 — March 2026*
