# CLI Tools: Rust Replacement Research
> Research method: Live web search + training knowledge (Feb 2026)

---

## Already Done Well

These tools have succeeded — widely adopted, production-quality, and demonstrate what makes a Rust CLI tool succeed:

| Tool | Replaces | Why It Succeeded |
|------|----------|-----------------|
| `ripgrep` | grep | 3-10x faster, .gitignore-aware, sane defaults |
| `fd` | find | Fast, user-friendly syntax, gitignore support |
| `bat` | cat | Syntax highlighting, git diff integration, paging |
| `eza` | ls | Colors, icons, git status, maintained (exa fork) |
| `delta` | diff | Side-by-side diffs, syntax highlighting |
| `bottom` / `btm` | htop/top | Visual, modern, cross-platform system monitor |
| `procs` | ps | Color output, keyword search, tree view |
| `dust` | du | Intuitive treemap-style disk usage |
| `sd` | sed | Simple find/replace, no regex foot-guns |
| `zoxide` | cd | Frecency-based directory jumping |
| `starship` | shell prompts | Fast, cross-shell, no config needed |
| `hyperfine` | time | Statistical benchmarking |
| `tokei` | cloc/wc | Fast code statistics |
| `xh` | curl/httpie | Friendly HTTP client |
| `tealdeer` | man/tldr | Fast tldr-pages client |
| `just` | make (task runner) | Simple, readable, justfiles |
| `navi` | cheatsheet tools | Interactive CLI cheatsheet |
| `choose` | cut (sometimes awk) | Human-friendly field selection |

**Pattern of success:** These tools win by doing one thing well, being opinionated about defaults, and being significantly faster than what they replace.

---

## Partial / Incomplete Coverage

Tools where a Rust version exists but is NOT production-quality or not widely adopted:

### awk
- `frawk` — JIT-compiled, CSV-native. Impressive benchmarks. But not a full POSIX awk replacement — missing some standard functions. HN thread (2022): positive reception but acknowledged as "not ready to replace awk in scripts". https://news.ycombinator.com/item?id=30343373
- `rawk` — experimental, barely used
- `choose` — covers the simple `awk '{print $N}'` case only

**Gap:** No drop-in POSIX-compatible awk replacement in Rust. `frawk` is the closest but needs a maintainer push to reach 1.0.

### sed
- `sd` — covers simple substitution well, but NOT a drop-in sed replacement (different syntax, missing line addressing, in-place multi-file, etc.)
- `red-sed` — experimental, low adoption

**Gap:** A true sed drop-in with POSIX compliance + Rust speed is unbuilt.

### GNU Coreutils (uutils project)
As of Feb 2026, the uutils project reports:
- **Coreutils**: Production level (cp, mv, ls, cat, echo, etc.)
- **Findutils** (find, xargs, locate): Getting close to completion
- **Diffutils**: Almost ready
- **procps** (top, ps, pgrep, pidof): In progress from GSoC 2024

Source: https://uutils.github.io/blog/2025-02-extending/

The GNU coreutils rewrite is happening — but as a compatibility project, not a "better" replacement. The individual user-facing gap is more `awk` and `sed` compatibility.

### tar
- No serious Rust tar replacement (beyond the `tar` crate for programming). Command-line `tar` drop-in: missing.

### watch
- No `watch` replacement in Rust. `watchexec` exists (watches files and re-runs commands) but doesn't replicate the simple "re-run a command every N seconds" interface.

### parallel (GNU Parallel)
- No GNU Parallel equivalent in Rust. GNU Parallel does distributed/parallel command execution — nothing replaces it in Rust.

### ImageMagick CLI
- `image-rs` crate covers the library side well
- `magick-rust` provides bindings to ImageMagick itself
- No standalone `convert`/`mogrify` equivalent in Rust
- Community interest exists: multiple Reddit threads asking for this

---

## Community Discussions

### What the community is asking for

From the **awesome-rewrite-it-in-rust** tracker (https://github.com/j-m-hoffmann/awesome-rewrite-it-in-rust), the most-upvoted gaps are:

1. **Full awk replacement** — appears in nearly every "what's missing" thread
2. **ImageMagick CLI** — image processing pipeline tool
3. **GNU Parallel equivalent** — parallel execution framework
4. **`watch` command** — simple, universal periodic re-runner
5. **`make` replacement** — `just` covers task runners but not full Makefile compatibility

From the **2024 Rust Annual Survey** (https://blog.rust-lang.org/2025/02/13/2024-State-Of-Rust-Survey-results/):
- CLI tooling is the #1 domain for Rust developers
- Profiling tools are underserved (70% use no profiling tools)
- Debugging experience still lags other ecosystems

**HN discussion on frawk (awk replacement):** "This is exactly what the Rust ecosystem needs — awk is one of the last holdouts that doesn't have a proper modern replacement" — https://news.ycombinator.com/item?id=30343373

**HN discussion on GNU coreutils:** Community divided on whether drop-in compatibility matters vs. just building better tools — https://users.rust-lang.org/t/im-shocked-the-rust-community-is-pushing-an-mit-licensed-rust-rewrite-of-gnu-coreutils/126110

---

## Failed / Stalled Attempts

| Project | Goal | Status | Why It Stalled |
|---------|------|--------|---------------|
| `exa` | ls replacement | Abandoned 2023, forked to `eza` | Single maintainer burnout |
| `ion shell` | bash replacement | Stalled ~2021 | Maintainer bandwidth, async shell design is hard |
| `rawk` | awk replacement | Abandoned | Never reached feature parity |
| `amber` | awk+sed | Minimal adoption | Not idiomatic, no community traction |

**Key lesson:** Single-maintainer projects for complex tools die. The ones that survive (ripgrep, fd, bat) had strong initial momentum, a clear niche, and community maintainers.

---

## Top Candidates for a New Rust CLI Project

### #1 — Full `awk` replacement (POSIX-compatible)
- **Gap:** No production-quality Rust awk. `frawk` is 80% there.
- **Demand:** Constant community requests. awk is in every sysadmin/devops workflow.
- **Rust advantage:** JIT compilation (frawk proves it), safer string handling, better Unicode
- **MVP scope:** POSIX awk subset + CSV/TSV mode = 3-4 months
- **Risk:** POSIX compatibility surface is large. Maintenance burden.

### #2 — `watch` replacement with superpowers
- **Gap:** No modern `watch` in Rust
- **Demand:** `watch` is used daily by every developer
- **Rust advantage:** Cross-platform (watch is Linux-only), lower overhead
- **Unique angle:** Add: color diff output, JSON/structured mode, conditional alerts
- **MVP scope:** 2-3 weeks for basic watch, 6-8 weeks for full feature set
- **Risk:** Low — small surface area, clear spec

### #3 — GNU Parallel equivalent
- **Gap:** GNU Parallel has no Rust alternative
- **Demand:** Used extensively in HPC, CI pipelines, batch processing
- **Rust advantage:** Memory safety for concurrent process management
- **MVP scope:** Core parallelism + input splitting = 4-6 weeks
- **Risk:** GNU Parallel has complex features (SSH distribution, etc.)

### #4 — ImageMagick CLI replacement
- **Gap:** `image-rs` crate is good but no CLI tool
- **Demand:** ImageMagick is infamous for CVEs and memory bugs
- **Rust advantage:** Every single ImageMagick CVE is a memory safety bug
- **MVP scope:** convert, resize, format conversion = 6-8 weeks
- **Risk:** ImageMagick supports hundreds of formats. Start small.

### #5 — `sed` drop-in replacement
- **Gap:** `sd` is good but NOT a sed drop-in
- **Demand:** sed is used in millions of shell scripts
- **Rust advantage:** No regex foot-guns, better error messages, faster
- **MVP scope:** Core POSIX sed + s command + line addressing = 4-6 weeks
- **Risk:** Full POSIX sed is surprisingly complex (branching, labels, etc.)

---

## Sources
- https://github.com/TaKO8Ki/awesome-alternatives-in-rust
- https://github.com/j-m-hoffmann/awesome-rewrite-it-in-rust
- https://uutils.github.io/blog/2025-02-extending/
- https://news.ycombinator.com/item?id=30343373 (frawk HN thread)
- https://users.rust-lang.org/t/im-shocked-the-rust-community-is-pushing-an-mit-licensed-rust-rewrite-of-gnu-coreutils/126110
- https://blog.rust-lang.org/2025/02/13/2024-State-Of-Rust-Survey-results/
- https://blog.jetbrains.com/rust/2026/02/11/state-of-rust-2025/
- https://itsfoss.com/rust-alternative-cli-tools/
- https://gist.github.com/sts10/daadbc2f403bdffad1b6d33aff016c0a
- https://dev.to/dev_tips/15-rust-cli-tools-that-will-make-you-abandon-bash-scripts-forever-4mgi
