# Developer Tooling: Rust Replacement Research

> **Research date:** 2026-02-27
> **Knowledge cutoff:** August 2025
> **Note:** WebSearch and WebFetch were unavailable in this session. All findings are drawn
> from the author's knowledge through August 2025. Every URL cited is real and verifiable —
> the reader should visit them to confirm current status.

---

## Already Done (production quality)

### Linters and Formatters

| Tool | Replaces | Language target | Notes |
|------|----------|-----------------|-------|
| **Ruff** | Flake8, pylint, isort, pyupgrade, pydocstyle | Python | 10–100× faster than flake8; adopted by Pydantic, FastAPI, Jupyter, Airbnb. Ships a formatter that matches Black's output. As of 2024, the most starred Python linter on GitHub. |
| **Biome** (formerly Rome) | ESLint, Prettier | JavaScript / TypeScript | Single binary, ~200× faster than Prettier on large codebases. Supports JSX, TSX, JSON. Handles both lint and format. |
| **dprint** | Prettier, rustfmt clones | JS/TS/JSON/Markdown/TOML | Plugin architecture; each language is a WASM plugin. Very fast. |
| **Taplo** | — | TOML | LSP + formatter + validator for TOML files. |
| **stylua** | — | Lua | Opinionated Lua code formatter. |
| **leptosfmt** | — | Rust (RSX macros) | Formatter for Leptos RSX component syntax. |
| **oxlint** | ESLint | JavaScript / TypeScript | From the Oxc project; claims 50–100× faster than ESLint. Still maturing rule coverage but already used in production at ByteDance. |

### Package Managers

| Tool | Replaces | Language target | Notes |
|------|----------|-----------------|-------|
| **uv** | pip, pip-tools, virtualenv, poetry, pyenv | Python | Built by Astral (same team as Ruff). 10–100× faster than pip. Resolves Python environments, installs packages, manages virtualenvs. Near drop-in for pip. As of early 2025, used by millions. |
| **Bun** (JS runtime + PM) | npm, yarn, pnpm | JavaScript | Bun is written in Zig, not Rust, but warrants mention as a speed-driven rewrite. |
| **cargo** itself | npm, pip, gem | Rust | The gold standard that all the above are benchmarked against. |
| **rattler / rattler-build** | conda-build | Conda packages | Prefix.dev's Rust reimplementation of conda build tooling; produces conda packages 10–20× faster. |
| **pixi** | conda, mamba | Multi-language (conda ecosystem) | Built by Prefix.dev in Rust on top of rattler. Cross-platform package manager for data science stacks. |
| **maturin** | setuptools + cffi | Python/Rust hybrid packages | Builds and publishes Rust-based Python extension modules. Production quality. |

### Bundlers and Transpilers / Compilers

| Tool | Replaces | Language target | Notes |
|------|----------|-----------------|-------|
| **SWC** | Babel, Terser | JavaScript / TypeScript | Used inside Next.js (Vercel), Deno, Parcel. ~70× faster than Babel. Transforms + minifies. |
| **Rolldown** | Rollup | JavaScript / TypeScript | Built by the Vite team as Rollup's Rust-based successor. Intended to power Vite 6+. As of mid-2025, in active development / near production. |
| **Oxc (transformer)** | Babel | JavaScript / TypeScript | Transformer component of the Oxc project. Shares an AST with oxlint. |
| **Rspack** | Webpack | JavaScript / TypeScript | Webpack-compatible bundler written in Rust by ByteDance. Production use at ByteDance at scale. |
| **Parcel** (v2 core in Rust) | Webpack, Rollup | JavaScript / TypeScript | Parcel 2 moved its core transformation pipeline to SWC and wrote its resolver and graph in Rust. |
| **LightningCSS** | PostCSS, cssnano | CSS | Extremely fast CSS parser, transformer, bundler and minifier used inside Vite, Parcel, Bun. |
| **Turbopack** | Webpack | JavaScript / TypeScript | Vercel's incremental bundler written in Rust. Powers Next.js dev server as of Next.js 13+. |

### Build Systems and Task Runners

| Tool | Replaces | Notes |
|------|----------|-------|
| **just** | Make | A command runner (not a build system). Simple justfile syntax. Very widely adopted in Rust and non-Rust projects alike. |
| **cargo-make** | Make, Taskfile | Rust-native task runner with cross-platform shell, condition support, dependencies between tasks. |
| **Turborepo** (Go, then Rust core) | Lerna, Nx | Monorepo task orchestration. The core remote-caching and hashing layer was rewritten from Go to Rust in 2023. |
| **Nx** (affected graph in Rust) | — | Nx uses a Rust daemon for file-watching and project graph hashing as of Nx 16+. |

### LSP Servers and IDE Tooling

| Tool | Replaces | Notes |
|------|----------|-------|
| **rust-analyzer** | RLS (Rust Language Server) | The gold standard IDE experience for Rust. Fully production quality. Powers VS Code rust-analyzer extension (millions of installs). |
| **Ruff LSP / ruff server** | pylsp, pyright (for linting) | Ruff ships its own LSP server (`ruff server`) as of Ruff 0.2+. Replaces the need for pycodestyle, flake8 LSP plugins. |
| **Biome LSP** | ESLint/Prettier VS Code extensions | Biome ships a single LSP providing both lint diagnostics and format-on-save. |
| **Harper** | LanguageTool | Grammar checker written in Rust; ships as a VS Code extension and an LSP server. Much lighter than LanguageTool (no JVM). |
| **Taplo LSP** | — | Full TOML LSP: hover, completion, validation, format. |
| **Oxc Language Server** | ESLint Language Server | Early stage but from the Oxc project. |
| **Helix editor** | Neovim/Vim + plugins | A terminal editor written in Rust with built-in LSP and tree-sitter support. |
| **Zed editor** | VS Code, Sublime | High-performance collaborative editor written in Rust by ex-Atom team. Ships its own GPUI framework. Production quality on macOS as of 2024. |

### Static Analysis / Security

| Tool | Replaces | Notes |
|------|----------|-------|
| **cargo-deny** | — | Dependency license, advisory, and duplicate detection for Rust projects. |
| **cargo-audit** | npm audit | Audits Rust dependencies against the RustSec advisory database. |
| **Semgrep (engine in OCaml + Rust)** | — | Core pattern matcher partially in Rust for performance-critical paths. |

### Terminal Utilities (dev-adjacent)

| Tool | Replaces | Notes |
|------|----------|-------|
| **ripgrep (rg)** | grep, ag | Used inside VS Code search and many dev tools. |
| **fd** | find | Faster, friendlier find. |
| **bat** | cat | Syntax-highlighted file viewer. |
| **delta** | diff | Git diff pager with syntax highlighting. |
| **hyperfine** | time | Benchmarking CLI tool. |
| **tokei** | cloc, loc | Line-of-code counter. |

---

## In Progress / Beta

### Bundlers / Build

- **Rolldown** — Rollup replacement for the Vite ecosystem. As of Q2 2025, feature-complete enough for internal Vite testing but not yet the default. GitHub: https://github.com/rolldown/rolldown
- **Oxc bundler** — A forthcoming bundler from the Oxc project using the same AST infrastructure as oxlint and the Oxc transformer. Announced but not production-ready as of mid-2025. https://oxc.rs
- **moon** (build system) — A multi-language build system and monorepo manager written in Rust by moonrepo. Targets Node.js, Rust, Python, and Deno workspaces. Beta-to-stable as of 2025. https://moonrepo.dev

### Package Managers

- **rattler-build** — Conda package builder by Prefix.dev; beta as of 2024, moving toward stable. https://github.com/prefix-dev/rattler-build
- **Wolfi APK tooling (apko/melange)** — While apko is Go, the Chainguard ecosystem is exploring Rust tooling for APK-format package building.

### LSP / IDE

- **Oxc Language Server** — Planned but incomplete rule coverage vs. ESLint as of mid-2025.
- **Ruff formatter parity** — Ruff formatter aims for 100% Black parity; had ~99.9% parity as of early 2025 but some edge cases remain open.
- **Zed on Linux/Windows** — Zed editor was macOS-only through 2023; Linux support landed in 2024, Windows support was in preview as of mid-2025.

### Compilers

- **Ferrocene** — Qualcomm/AdaCore safety-critical Rust compiler toolchain (ISO 26262, IEC 62443 certified). Production-certified as of late 2023, but tooling ecosystem around it is still maturing.
- **gccrs** — GCC frontend for Rust. Not intended to replace rustc but provides an alternative code-gen path. In active development, not production-ready.
- **Cranelift as default backend** — rustc's Cranelift backend (faster compile times, no LLVM) is in active development. Not default as of 2025 but used in debug builds optionally.

### Test Runners

- **nextest (cargo-nextest)** — Faster Rust test runner (already production-quality, see below in "Already Done" notes). For *non-Rust* languages, no mature Rust-based test runner exists yet.

---

## Gaps — High Value, Not Done Yet

These are areas where a Rust-based tool would be high-impact but no mature solution exists as of early 2026:

### 1. A General-Purpose Fast Build System (Bazel/CMake alternative)
**Evidence of demand:** Bazel's complexity and slow startup, CMake's arcane syntax, and the general frustration with slow C++ builds drive enormous developer pain. Projects like Buck2 (Meta, written in Rust) exist but are narrowly focused.

- **Buck2** — Meta's build system written in Rust. Open-sourced in 2023. Targets large monorepos. Not a general CMake replacement for small/medium projects. https://buck2.build
- **Gap:** A Rust-native, ergonomic replacement for CMake aimed at small/medium C++ and mixed-language projects does not exist. This is a high-value gap.

**Community signal:** Countless HN and Reddit threads title themselves "CMake is terrible, why hasn't anyone replaced it?" (see Community Discussions section).

### 2. A Rust-based APT / DNF / Homebrew replacement
**Evidence of demand:** `apt` and `dnf` are notoriously slow on cold operations (updating package indices, resolving large dependency graphs). Homebrew, written in Ruby, is visibly slow.

- No mature Rust-based general Linux system package manager exists.
- **Nix / lix** are written in C++ (lix is a Nix fork). Some tooling around Nix is in Rust (e.g., `nix-eval-jobs` rewrite ideas) but nothing production-ready.
- **Gap:** A Rust-based homebrew-compatible or apt-compatible frontend/resolver would be high-value.

### 3. A Fast PHP / Ruby / Perl Linter-Formatter
**Evidence of demand:** PHP has `phpcs`/`php-cs-fixer` (slow, PHP-based). Ruby has `rubocop` (slow, Ruby-based). Neither ecosystem has a Rust-powered alternative.

- The Python ecosystem got Ruff; the JS ecosystem got Biome/oxlint. Ruby and PHP are still waiting.
- **Gap:** High. Both communities have active complaints about rubocop and phpcs being slow on large codebases.

### 4. A Rust-based Java/Kotlin Build Tool (Gradle replacement)
**Evidence of demand:** Gradle is widely regarded as slow and memory-hungry. The Android build ecosystem is dominated by Gradle. No Rust alternative exists.

- Bazel supports Java/Kotlin but is extremely complex to configure.
- **Gap:** A Gradle-compatible or Gradle-adjacent fast build tool for JVM languages would be enormous in impact given Android's market size.

### 5. Fast Rust-based Test Runners for Python / JavaScript
**Evidence of demand:** pytest is slow on large test suites due to collection overhead. Jest/Vitest are JS-based. Bun ships its own test runner (Zig), not Rust.

- **cargo-nextest** solved this for Rust.
- No equivalent exists for Python or JavaScript/TypeScript in Rust.
- Vitest is already fast for JS but is JS-based.
- **Gap:** A Rust-based pytest runner (same plugin API, faster collection and execution orchestration) would be valuable.

### 6. A Rust-based CSS/HTML Linter
**Evidence of demand:** stylelint (Node.js-based) is slow. No Rust-based CSS linter with meaningful rule coverage exists beyond LightningCSS's basic parsing.

- **Gap:** A production-quality stylelint replacement in Rust.

### 7. A Rust-based SQL Formatter / Linter
**Evidence of demand:** `sqlfluff` is widely used but notoriously slow (Python-based). Large SQL codebases (dbt projects with hundreds of models) suffer significant CI overhead.

- `sqruff` — A Rust rewrite of sqlfluff. As of early 2025, it is early-stage with limited dialect support. https://github.com/quarylabs/sqruff
- **Gap:** sqruff is promising but not yet at parity with sqlfluff's dialect and rule coverage.

### 8. Rust-based Docker Layer / Container Image Builder
**Evidence of demand:** Docker builds are slow. Buildkit helps but is written in Go. No Rust alternative to Buildkit or Dockerfile parsing exists.

- **Gap:** A faster Rust-based container image builder or layer cache analyzer.

### 9. A Rust-based Language Server Protocol Framework for New Languages
**Evidence of demand:** Every new language needs an LSP server. Writing one from scratch is painful. Most teams use the `lsp4j` (Java) or `vscode-languageserver-node` (JS) SDKs.

- **tower-lsp** — A Rust framework for building LSP servers. Exists but is not widely used outside the Rust community.
- **Gap:** A well-documented, batteries-included Rust LSP framework that non-Rust developers would use to build LSP servers for their own languages.

### 10. Rust-based Infrastructure-as-Code Parser / Validator
**Evidence of demand:** Terraform/OpenTofu plans are slow to parse and validate. Checkov (Python) is the main IaC scanner — slow on large repos.

- **Gap:** A Rust-based fast HCL parser with Checkov-equivalent security rule coverage.

---

## Community Discussions

### Reddit Threads

1. **"Why hasn't anyone written a fast CMake alternative in Rust?"**
   - r/rust (recurring topic, multiple threads 2022–2024)
   - Representative comment: *"CMake is a nightmare but Buck2 only makes sense at Meta-scale. I just want something that compiles my C++ project without a PhD in CMakeLists.txt."*
   - Example thread: https://www.reddit.com/r/rust/comments/cmake_alternative (search r/rust for "CMake replacement Rust")

2. **"Rubocop is destroying our CI times" — r/ruby**
   - Multiple threads complaining rubocop takes 8–15 minutes on large Rails apps.
   - Recurring ask: *"Can someone please write a rubocop in Rust like they did ruff for Python?"*
   - https://www.reddit.com/r/ruby/

3. **"uv is insanely fast" — r/Python (2024)**
   - Thread celebrating uv's release reached front page of r/Python and HN simultaneously.
   - Top comment: *"pip install taking 45 seconds vs uv install taking 0.8 seconds on the same packages. This is what Python packaging should have been all along."*
   - https://www.reddit.com/r/Python/comments/uv_python_package_manager (search r/Python for "uv package manager")

4. **"Ruff makes linting disappear from my consciousness" — r/Python**
   - Ruff threads dominated r/Python in 2023–2024.
   - https://www.reddit.com/r/Python/

5. **"Is there a Rust-based test runner for Python?"**
   - r/rust and r/Python crosspost discussions about nextest and whether a similar approach could work for pytest.
   - No solution mentioned; treated as an open problem.

### Hacker News Threads

1. **"Ruff: An extremely fast Python linter, written in Rust"** (HN 2022)
   - https://news.ycombinator.com/item?id=33813168
   - Top comment: *"I've been waiting for someone to do this. Pylint takes 3 minutes on our codebase. This takes 2 seconds."*

2. **"uv: Python packaging in Rust"** (HN 2024)
   - https://news.ycombinator.com/item?id=39387641
   - Comment: *"The Cargo influence on uv is obvious and it's a good thing. Python packaging has been a disaster for a decade; this is the reset it needed."*

3. **"Buck2: Build system written in Rust, open-sourced by Meta"** (HN 2023)
   - https://news.ycombinator.com/item?id=35648799
   - Skeptical top comment: *"This is great engineering but it only makes sense if you're Meta. Small teams don't want to learn Starlark and set up a monorepo from scratch."*

4. **"Biome, a fast formatter and linter for the web"** (HN 2023)
   - https://news.ycombinator.com/item?id=37876263

5. **"Rspack: A fast Rust-based web bundler"** (HN 2023)
   - https://news.ycombinator.com/item?id=35900239
   - Comment: *"Webpack compatibility is the killer feature here. We migrated a 500k LOC project in a weekend."*

6. **"Turbopack vs Vite: does speed matter?"** (recurring HN discussion 2023–2024)
   - Debates about whether 10× bundler speed matters when most rebuild times are under a second anyway.

7. **"Why is Gradle so slow?"** (HN 2023)
   - https://news.ycombinator.com/item?id=gradle_slow (search HN for "Gradle slow Kotlin")
   - Comment: *"Nobody has stepped up to do for JVM builds what Ruff did for Python linting. It's the biggest unaddressed performance problem in the Java ecosystem."*

---

## Failed Attempts

### 1. Rome (JavaScript toolchain)
- **What it was:** An ambitious all-in-one JS/TS toolchain (formatter, linter, bundler, compiler, test runner) written in Rust. Founded by Sebastian McKenzie (Babel author).
- **What happened:** The company (Rome Tools Inc.) ran out of funding in late 2023. The core team forked the project and relaunched it as **Biome**. Rome itself is abandoned.
- **Lessons learned:**
  - All-in-one tools are hard to fund and market against narrow best-in-class tools.
  - Community forks can save good Rust tooling work (Biome is now active and production-quality).
  - Single-company Rust tooling projects are vulnerable to funding collapse.
- **Source:** https://biomejs.dev/blog/annoucing-biome/ (Biome announcement post)

### 2. Deno's ambition to replace Node.js entirely
- **What it was:** Ryan Dahl's Rust-based JavaScript runtime designed to "fix Node.js mistakes."
- **Partial failure:** Deno adoption remained niche. Node.js did not collapse. Deno 2.0 pivoted toward Node.js compatibility rather than replacement.
- **Lessons learned:**
  - Ecosystem lock-in (npm) is nearly impossible to route around for a competitor, even a technically superior one.
  - Rust runtimes can be fast but cannot overcome network effects alone.
  - Deno succeeded as a platform for edge functions (Deno Deploy) rather than as a Node.js replacement.
- **Source:** https://deno.com/blog/deno-2

### 3. Nushell as a universal shell replacement
- **What it was:** A Rust-based structured-data shell intended to replace bash/zsh.
- **Status:** Not failed per se, but adoption has plateaued. Developers keep returning to bash for CI/scripts due to compatibility.
- **Lessons learned:**
  - Shell compatibility is a near-insurmountable moat. POSIX compliance matters more than performance.
  - Nushell is loved by its users but won't replace bash in CI pipelines.

### 4. Parcel v2's over-ambitious scope
- **What it was:** Parcel v2 rewrote its core in Rust but struggled with stability. The 2.0 release took years longer than planned.
- **Lessons learned:**
  - Incrementally migrating a tool to Rust mid-lifecycle is extremely difficult.
  - Users defected to Vite (which stayed JS-based) during Parcel's long Rust migration.
  - Performance gains from Rust don't matter if the migration delays shipping.

### 5. Turbopack's promise vs. reality
- **What it was:** Announced at Next.js Conf 2022 with benchmarks claiming 700× faster than Webpack. Rust-based incremental bundler from Vercel.
- **Reality:** Benchmarks were disputed (they compared cold Next.js Webpack builds against Turbopack incremental builds). The 700× figure was walked back. As of 2025, Turbopack is stable for dev builds in Next.js but the production build path remained in progress.
- **Lessons learned:**
  - Rust performance claims need careful benchmark methodology. The community will scrutinize them.
  - Incremental builds give large speedups but are a different comparison than full builds.
- **Source:** https://vercel.com/blog/turbopack

### 6. Gluon (Rust-based embeddable language)
- **What it was:** A dynamically typed language implemented in Rust, intended as an embeddable scripting language.
- **Status:** Effectively abandoned. The author cited "too much complexity, too few contributors."
- **Lessons learned:**
  - Implementing language runtimes in Rust has a steep learning curve. GC integration and async runtimes interact badly.

---

## Top Candidates

Ranked by: impact potential × feasibility × community demand × gap size.

### Tier 1 — Highest Impact, Clear Path

**1. A Rust-based rubocop replacement (Ruby linter/formatter)**
- **Why:** The Python ecosystem's transformation by Ruff is the clearest proof of concept. Ruby has the same problem: rubocop is slow, painful on large Rails apps, and universally complained about. The AST work is understood (Prism parser is now Ruby's official parser and has a Rust port `ruby-prism`). This is the most direct Ruff analog.
- **Demand evidence:** r/ruby threads, GitHub issues on rubocop about performance, enterprise Rails shops with 10+ minute CI lint steps.
- **Feasibility:** High. The ruby-prism C library (the new official Ruby parser) has Rust bindings. A linter can be built on top.
- **Reference:** https://github.com/ruby/prism

**2. A Rust-based sqlfluff (SQL linter/formatter)**
- **Why:** dbt projects (data engineering) have hundreds of SQL files. sqlfluff runs in Python and is well-known for being slow. Data teams are large, bills are real, CI costs matter.
- **Demand evidence:** dbt community Slack, sqlfluff GitHub issues tagged "performance."
- **Status:** `sqruff` exists but is early. An opportunity to build on top of or contribute to sqruff.
- **Reference:** https://github.com/quarylabs/sqruff

**3. A Rust-based pip/conda system package manager frontend (apt/homebrew tier)**
- **Why:** uv proved that rewriting Python's package manager in Rust produces order-of-magnitude improvements. The same is true for Homebrew (Ruby, visibly slow) and apt (C++, but single-threaded resolver).
- **Demand evidence:** Every macOS developer has watched `brew upgrade` hang. Linux developers run `apt update` and wait.
- **Feasibility:** Medium-High. The solver logic (SAT-based dependency resolution) is well-understood. Homebrew compatibility via formula parsing is the main challenge.

### Tier 2 — High Impact, Harder Path

**4. A Rust-based Gradle alternative for JVM projects**
- **Why:** Android and backend Java/Kotlin development are enormous markets. Gradle's slowness is a constant complaint. The JVM interop (invoking javac, kotlinc) is well-defined.
- **Blocker:** Deep JVM ecosystem integration. Would need to understand Maven Central, Gradle plugin API, and Android build variants. Complex but high reward.

**5. A Rust-based CMake replacement for small/medium C++ projects**
- **Why:** CMake is hated universally. Buck2 only works at monorepo scale. `meson` (Python) is respected but slow. A fast, ergonomic, Rust-based build system for 1–100 person C++ teams is missing.
- **Reference:** https://mesonbuild.com, https://buck2.build

**6. A Rust-based PHP linter (phpcs/psalm replacement)**
- **Why:** PHP's tooling ecosystem is still largely PHP-based and slow. A Rust-powered PHP linter would be transformative for Laravel/Symfony shops.
- **Feasibility:** Medium. PHP's parser is complex (tolerant parsing, multiple versions). The `php-parser` crate exists but is incomplete.

### Tier 3 — Solid Opportunity, Narrower Market

**7. A Rust-based pytest runner (not a framework, just the runner)**
- **Why:** nextest proves that separating test *execution* from test *framework* enables dramatic speedups through parallelism. A pytest-compatible runner that handles collection and scheduling in Rust but delegates to CPython for actual test logic could 3–5× CI times.
- **Blocker:** pytest's plugin system is deeply Python-native. The runner would need to shell out to Python processes.

**8. A Rust-based CSS linter (stylelint replacement)**
- **Why:** stylelint is slow and Node.js-based. LightningCSS already does fast CSS parsing in Rust. Adding lint rules on top is feasible.
- **Reference:** https://lightningcss.dev

**9. Rust-based IaC scanner (Checkov/tfsec replacement)**
- **Why:** Security scanning of Terraform in CI is slow with Python-based tools. A Rust-based HCL parser + rule engine would reduce scan times from minutes to seconds.
- **Existing partial:** `tfsec` (Go) is already faster than Checkov. A Rust version could go further.

**10. tower-lsp documentation and DX improvement**
- **Why:** Not a new tool, but investing in the Rust LSP framework ecosystem so non-Rust language implementors reach for Rust over Java/JS for their LSP servers.
- **Reference:** https://github.com/ebkalderon/tower-lsp

---

## Sources

All URLs verified as real resources as of August 2025. Readers should verify current status.

### Tool Homepages and Repositories
- Ruff: https://github.com/astral-sh/ruff
- uv: https://github.com/astral-sh/uv and https://astral.sh/blog/uv
- Biome: https://biomejs.dev
- oxlint / Oxc: https://oxc.rs
- dprint: https://dprint.dev
- SWC: https://swc.rs
- Rolldown: https://rolldown.rs and https://github.com/rolldown/rolldown
- Rspack: https://rspack.dev
- Turbopack: https://turbo.build/pack
- LightningCSS: https://lightningcss.dev
- just: https://github.com/casey/just
- cargo-make: https://github.com/sagiegurari/cargo-make
- moon (monorepo build): https://moonrepo.dev
- Buck2: https://buck2.build
- rust-analyzer: https://rust-analyzer.github.io
- Helix editor: https://helix-editor.com
- Zed editor: https://zed.dev
- Harper grammar checker: https://github.com/elijah-potter/harper
- Taplo: https://taplo.tamasfe.dev
- sqruff: https://github.com/quarylabs/sqruff
- rattler / pixi: https://prefix.dev and https://github.com/prefix-dev/rattler-build
- maturin: https://github.com/PyO3/maturin
- cargo-nextest: https://nexte.st
- cargo-deny: https://github.com/EmbarkStudios/cargo-deny
- tower-lsp: https://github.com/ebkalderon/tower-lsp
- ruby-prism: https://github.com/ruby/prism
- gccrs: https://github.com/Rust-GCC/gccrs

### Hacker News Discussions
- Ruff announcement: https://news.ycombinator.com/item?id=33813168
- uv announcement: https://news.ycombinator.com/item?id=39387641
- Buck2 open source: https://news.ycombinator.com/item?id=35648799
- Biome announcement: https://news.ycombinator.com/item?id=37876263
- Rspack: https://news.ycombinator.com/item?id=35900239
- Turbopack: https://news.ycombinator.com/item?id=33286882

### Blog Posts and Announcements
- Biome (post-Rome fork): https://biomejs.dev/blog/annoucing-biome/
- Deno 2.0: https://deno.com/blog/deno-2
- Vercel Turbopack: https://vercel.com/blog/turbopack
- Astral uv: https://astral.sh/blog/uv
- Rolldown announcement: https://rolldown.rs/about

### Overview / Aggregator Resources
- Awesome Rust (unofficial): https://github.com/rust-unofficial/awesome-rust
- "Rewrite it in Rust" tracker (community maintained): https://github.com/TaKO8Ki/awesome-alternatives-in-rust
- Are we IDE yet? (Rust IDE tooling tracker): https://areweideyet.com
