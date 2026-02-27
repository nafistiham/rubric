# Language Runtimes & Interpreters: Rust Research

> Research date: 2026-02-27
> Note: WebSearch and WebFetch were unavailable in this session. All findings are drawn from
> training-data knowledge (cutoff August 2025), citing canonical project URLs and known
> community discussions. Treat version numbers / status claims as accurate as of mid-2025.

---

## Existing Rust Runtimes / Interpreters

### Python — RustPython

**Repo:** https://github.com/RustPython/RustPython
**Status:** Active but not production-ready (as of mid-2025).

RustPython is a Python 3 interpreter written entirely in Rust. It targets CPython compatibility
but lags significantly behind. Key facts:

- Targets Python 3.11/3.12 semantics but is missing large swaths of the standard library.
- The C extension API (CPython's `Python.h`) is NOT implemented. This means NumPy, pandas,
  scikit-learn, and nearly all scientific/data packages **cannot run**.
- Performance on pure-Python benchmarks is often slower than CPython, not faster — RustPython
  currently uses a pure AST-walking / bytecode-interpreting approach with no JIT.
- Does compile to WebAssembly; its demo at https://rustpython.github.io runs Python in the
  browser. This is its strongest differentiator vs CPython.
- The project has acknowledged it is primarily a research/learning vehicle and a WASM showcase,
  not a CPython replacement.

**Where it falls short:**
1. No C extension support (kills 90 %+ of the ecosystem).
2. Missing stdlib modules (e.g., `multiprocessing`, `ctypes`, large parts of `asyncio`).
3. No JIT compiler — slower than CPython on CPU-bound code.
4. No production deployments documented.
5. Small core-contributor pool; many issues open for years.

**Verdict:** Interesting proof-of-concept and WASM embedding story. Not a drop-in CPython
replacement. Unsuitable for production Python workloads.

---

### JavaScript / TypeScript — Deno

**Site:** https://deno.com
**Repo:** https://github.com/denoland/deno
**Status:** Production-ready. v2.x is stable (2024–2025).

Deno is a JavaScript/TypeScript runtime built on V8 + Tokio (Rust async runtime). It does NOT
rewrite V8 in Rust — it embeds V8 and wraps it with a Rust event loop and standard library.

**What Deno's Rust foundation enables that Node cannot easily do:**

1. **Single-binary deployment.** `deno compile` bundles your JS/TS app into a self-contained
   native executable. Node has no equivalent (pkg/nexe are third-party hacks).
2. **Secure-by-default sandboxing.** Permissions (`--allow-read`, `--allow-net`, etc.) are
   enforced at the Rust layer, not in JS. Node's permission model arrived late (v22) and is
   less ergonomic.
3. **Unified toolchain.** Deno ships a formatter, linter, test runner, bundler, and LSP all in
   one binary, implemented in Rust (dprint, oxc lint, etc.).
4. **First-class TypeScript.** No `tsc` invocation needed; Deno's Rust layer handles
   transpilation via SWC.
5. **Tokio-powered async.** Deno's async I/O uses Tokio directly, enabling fine-grained control
   over the event loop that is impossible to retrofit into Node's libuv.
6. **Deno Deploy / edge workers.** Deno's architecture made building a V8-isolate-per-request
   edge platform (like Cloudflare Workers) straightforward. Node's architecture makes this hard.

**Limitation:** V8 is still the JS engine. Deno does not rewrite JS execution itself.

---

### JavaScript — Boa

**Repo:** https://github.com/boa-dev/boa
**Status:** Experimental / early production.

Boa is a pure-Rust JavaScript engine (lexer, parser, bytecode compiler, VM). It is NOT
embedding V8/SpiderMonkey — it implements ECMAScript from scratch in Rust.

- Passes a growing subset of Test262 (the official ECMAScript conformance suite).
- As of 2024, ~80–85 % of Test262 passes.
- Primarily useful for embedding JS in Rust applications where you want a small, auditable
  engine without pulling in V8's 10 MB+ binary.
- No JIT yet; significantly slower than V8/SpiderMonkey for real workloads.
- Used by some game engines and config-DSL systems.

---

### JavaScript — Nova (experimental)

**Repo:** https://github.com/trynova/nova
**Status:** Pre-alpha research project (2024–2025).

Nova is an attempt at a next-generation ECMAScript engine in Rust with a data-oriented design
(struct-of-arrays layout for the GC heap). Inspired by SerenityOS's LibJS. Not usable for
production. Fascinating architecture research.

---

### Lua — mlua / Piccolo / Hematite

- **mlua** (https://github.com/khvzalenko/mlua): Rust bindings to the official Lua 5.x / LuaJIT
  C library via FFI. The most production-ready way to embed Lua in Rust apps. Not a reimplementation.
- **Piccolo** (https://github.com/kyren/piccolo): A pure-Rust Lua 5.4 interpreter with a novel
  stackless/async-friendly design. Early but technically innovative — avoids the classic problem
  of Lua's C stack conflicting with Rust's ownership. Status: alpha.
- **Hematite / lua-in-rust projects:** Multiple toy/learning implementations exist on crates.io;
  none are production-grade.

---

### Ruby — Artichoke

**Repo:** https://github.com/artichoke/artichoke
**Status:** Alpha / experimental.

Artichoke is a Ruby runtime written in Rust that embeds mruby (the lightweight C Ruby VM) but
aims to replace mruby internals with Rust components over time. Think of it as a gradual
mruby-in-Rust migration. Not compatible with MRI C extensions. No production users documented.

---

### Ruby — YJIT (Not Rust-based, but relevant)

Ruby's YJIT JIT compiler (shipping since Ruby 3.1) was **rewritten from C to Rust** by
Shopify engineers in Ruby 3.2 (2022). This is the highest-profile "rewrite a language runtime
component in Rust" success story:

- The Rust YJIT is now the canonical implementation, merged into ruby/ruby.
- It improved maintainability, safety, and performance vs the prior C version.
- Demonstrates Rust's viability for JIT backends — not the whole VM, but a critical hot path.
- **Key takeaway:** Rust can be integrated into existing C runtimes incrementally, not just
  as a full rewrite.

---

### PHP — Hack (HHVM) and alternatives

No serious Rust-based PHP runtime exists. HHVM (Facebook, written in C++) remains the main
alternative to PHP's Zend engine. This is an open gap.

---

### Perl

No Rust Perl runtime. Perl's ecosystem is deeply tied to XS (C extensions). Theoretical
reimplementation would face the same C-extension wall as RustPython.

---

### SQL Engines

- **Datafusion** (https://github.com/apache/datafusion): Apache Arrow DataFusion is a
  Rust-native SQL query engine. Production-ready, used by InfluxDB 3.0, Delta Lake
  implementations, and others. Not a "language runtime" in the traditional sense but is a
  complete SQL execution runtime.
- **GlareDB** and **Ballista** are also Rust SQL engines.
- **DuckDB** has a Rust API but its core is C++.

---

### Scheme / Lisp

- **Steel** (https://github.com/mattwparas/steel): A Scheme-like scripting language + runtime
  entirely in Rust. Embeddable. Active development 2023–2025. Supports async. Interesting for
  Rust app scripting use cases.

---

### Rhai (Scripting DSL for Rust)

**Repo:** https://github.com/rhaiscript/rhai
**Status:** Production-ready for embedding.

Rhai is a purpose-built scripting language for embedding in Rust applications. It is NOT an
implementation of an existing language. Syntax is JavaScript-like. Designed for game scripting,
config evaluation, and plugin systems. Has WASM support. Widely used in the Rust ecosystem for
"scriptable Rust apps."

---

### R — Not present

No Rust R runtime exists. R is deeply tied to FORTRAN/BLAS and its own C extension system
(Rcpp). Reimplementing would be a massive undertaking with unclear demand outside of scientific
computing.

---

### Java / JVM

#### Espresso (GraalVM)

GraalVM's Espresso project is a JVM-on-JVM (Java interpreter written in Java, running on
GraalVM's Truffle framework). This is not Rust-based but is the most notable JVM alternative.

#### Rust-based JVM attempts

- **RJVM** and similar toy projects exist on GitHub, implementing basic class loading and
  bytecode execution in Rust.
- No production-grade Rust JVM exists.
- The JVM spec is enormous (class file format, GC requirements, JNI for native calls, JIT
  semantics). Building a production JVM from scratch in any language is a multi-decade effort
  — this is why only IBM, Oracle, Eclipse (OpenJ9), and Azul have ever shipped one.

**Gap with demand:** JVM performance and startup time are persistent complaints. GraalVM's
native-image partially solves AOT startup. A Rust JVM targeting fast startup / small footprint
for serverless JVM workloads would be differentiated — but immensely difficult.

---

### .NET / CLR

- **Mono** is C, maintained by Microsoft.
- No Rust CLR implementation exists. The CLR is even more complex than the JVM (COM interop,
  P/Invoke, unsafe memory model, etc.).

---

### WebAssembly Runtimes (see dedicated section below)

---

## WebAssembly Ecosystem

### Wasmtime

**Repo:** https://github.com/bytecodealliance/wasmtime
**Org:** Bytecode Alliance (Mozilla, Fastly, Intel, Microsoft, Arm, etc.)
**Status:** Production-ready. v1.0 released September 2022. Active v19–20+ as of 2025.

Wasmtime is the reference Rust-based WebAssembly runtime. Features:

- Built on **Cranelift** (a Rust-native code generator / JIT backend).
- Implements WASI (WebAssembly System Interface) — the POSIX-like syscall layer for WASM.
- Component Model support (WASM components, WIT interface definitions).
- Used in: Fastly Compute@Edge, various serverless platforms, Fermyon Spin.
- Security model: each WASM instance is sandboxed by design.
- Language bindings: Rust, Python, C, .NET, Go.
- Performance: near-native for CPU-bound workloads post-JIT compilation.

### Wasmer

**Repo:** https://github.com/wasmerio/wasmer
**Status:** Production-ready. Commercially backed (Wasmer Inc.).

Wasmer competes directly with Wasmtime. Key differentiators:

- Multiple compiler backends: Cranelift, LLVM, and Singlepass (for fast compile, lower peak
  performance — good for short-lived functions).
- WAPM (WebAssembly Package Manager) ecosystem.
- `wasmer run` CLI — download and run WASM packages from the registry.
- Language bindings broader than wasmtime: Python, JavaScript, Ruby, Java, Go, PHP, R, etc.
- Universal WASM binaries: compile once, run on Linux/macOS/Windows/WASI.

**Wasmer vs Wasmtime:**

| Dimension            | Wasmtime                        | Wasmer                              |
|----------------------|---------------------------------|-------------------------------------|
| Backing org          | Bytecode Alliance (non-profit)  | Wasmer Inc. (VC-backed)             |
| Compiler backends    | Cranelift only                  | Cranelift, LLVM, Singlepass         |
| Standards focus      | Spec-compliant, conservative    | Move fast, broader ecosystem        |
| WASI support         | First-class                     | Good                                |
| Component Model      | Reference implementation        | In progress                         |
| Edge deployments     | Fastly                          | Wasmer Edge                         |

### WasmEdge

**Repo:** https://github.com/WasmEdge/WasmEdge (C++ core, Rust bindings)
**Status:** Production. CNCF sandbox project.

WasmEdge is primarily C++ with Rust bindings. It targets AI/ML inference at the edge — has
WASM extensions for ONNX, TensorFlow Lite. Not purely a Rust runtime.

### Lunatic

**Repo:** https://github.com/lunatic-solutions/lunatic
**Status:** Experimental (2022–2024). Development slowed.

Lunatic is a Rust-based WASM runtime for building Erlang-style actor systems. Each actor is a
WASM module. Preemptive scheduling, message passing, process isolation — all at the WASM level.
Fascinating concept: "Erlang's concurrency model but your code compiles to WASM." Status as of
2025 is uncertain — the main maintainer reduced activity.

### Spin (Fermyon)

**Repo:** https://github.com/fermyon/spin
**Status:** Production. Fermyon's serverless WASM framework.

Spin is built on Wasmtime. It provides a higher-level framework for writing serverless functions
in any WASM-targeting language (Rust, Python via componentize-py, JS via js2wasm, Go, etc.).
Fermyon Cloud deploys them. This is the current leading example of "WASM as a deployment unit."

---

## Scripting Language Embeds (Libraries for Embedding Runtimes in Rust Apps)

### PyO3 — Python ↔ Rust FFI

**Repo:** https://github.com/PyO3/pyo3
**Status:** Production-ready. The de-facto standard.

PyO3 lets you:
1. Write Python extension modules in Rust (exported as `.so` / `.pyd`).
2. Call Python from Rust (embedding CPython).

It does NOT reimplement Python; it wraps the real CPython via its stable C API. Used by:
- **Polars** (DataFrame library — Rust core, Python bindings via PyO3)
- **Ruff** (Python linter written in Rust, uses PyO3 for the Python API surface)
- **maturin** (build tool: packages Rust-as-Python-extension)

This is the dominant model for "Python + Rust" in production — not a Rust Python interpreter,
but Rust code callable from Python.

### mlua — Lua ↔ Rust

**Repo:** https://github.com/khvzalenko/mlua
**Status:** Production-ready.

Binds the official Lua 5.1/5.2/5.3/5.4 and LuaJIT C libraries into Rust. Async-aware (works
with Tokio). Used for embedding Lua scripting in Rust game engines and servers. The standard
approach for "I want to script my Rust app with Lua."

### Rquickjs — QuickJS in Rust

**Repo:** https://github.com/DelSkayn/rquickjs
**Status:** Active, production-usable.

Binds Fabrice Bellard's QuickJS (a small, embeddable C JS engine) into Rust. QuickJS is
ES2020-compliant, has a small binary size, and supports AOT bytecode. Good for embedding JS
scripting in Rust apps without pulling in V8.

Used by: Deno itself uses a Rust-V8 binding (`rusty_v8`), but for lighter use cases rquickjs
is popular.

### Rusty_v8 — V8 in Rust

**Repo:** https://github.com/denoland/rusty_v8
**Status:** Production-ready (Deno depends on it).

Rust bindings for Google's V8 engine. Deno uses this as its JS execution layer. High-quality
but complex to use standalone — most users use Deno's higher-level APIs.

---

## What's Missing / Underdeveloped

### 1. A production-grade Rust Python interpreter with C-extension support

**Demand evidence:**
- RustPython GitHub has 19k+ stars (as of mid-2025), indicating massive interest.
- Repeated HN/Reddit threads ask "when will RustPython support NumPy?"
- The WASM Python niche is somewhat served, but a fast, C-extension-compatible Python runtime
  in Rust does not exist.

**Why it's hard:** CPython's C API (`PyObject*`, reference counting, GIL semantics) is deeply
entangled. Reimplementing it means reimplementing a 30-year-old ABI. PyPy tried and partially
succeeded (cpyext), but it's slow and incomplete.

**Opportunity:** A Rust runtime that implements the CPython stable ABI as a compatibility shim
could run C extensions without source changes. No project has credibly attempted this.

---

### 2. A Rust-based JVM with fast startup for serverless

**Demand evidence:**
- AWS Lambda cold start with JVM is 1–3 s vs <100 ms for Go/Rust.
- GraalVM native-image partially solves this but has reflection/classloading caveats.
- Hundreds of StackOverflow threads and AWS blog posts discuss JVM cold start pain.

**Why it's hard:** The JVM spec is enormous. JNI (native calls) alone is a multi-year project.
The GC must be generational and concurrent. This is a decade-long investment.

**Opportunity size:** Moderate. GraalVM native-image is improving and may close the gap before
a Rust JVM could be built.

---

### 3. A fast Rust Ruby runtime

**Demand evidence:**
- Ruby's reputation for slowness is a long-standing complaint (Reddit r/ruby, HN).
- Rails shops constantly profile and optimize to work around MRI's GIL and speed.
- YJIT (Rust JIT in MRI) is a partial answer but is tied to MRI's C codebase.

**Opportunity:** A clean-room Rust Ruby runtime with a modern GC (e.g., MMTK) and a JIT could
be highly impactful. Ruby's C extension ecosystem (gems with `.so` files) would be a blocker,
but less severe than Python's because fewer critical Ruby gems have C extensions.

---

### 4. A Perl runtime in Rust

Low demand. Perl is declining. Not a priority opportunity.

---

### 5. R runtime in Rust

Moderate academic/scientific interest. R is slow for non-vectorized code. An R runtime in Rust
with fast interpreted loops would be useful. Practically zero activity in this space.

---

### 6. PHP runtime in Rust

**Demand evidence:**
- PHP runs ~75 % of the web (WordPress, Drupal, etc.).
- FrankenPHP (Go-based PHP embedding) showed there is appetite for non-C PHP runtimes.
- No Rust PHP runtime attempt exists.

**Why it's hard:** PHP's extension model (`Zend Engine` API) is complex. But PHP's core
semantics are simpler than Python's — no GIL, simpler object model.

**Opportunity:** High impact if targeting WordPress/Drupal hosting at scale. Niche but real.

---

### 7. Bash / Shell runtime in Rust

**Existing work:**
- **nsh** (https://github.com/nuta/nsh): A Rust shell with some bash compatibility. Incomplete.
- **nu** (https://github.com/nushell/nushell): NuShell is a new shell paradigm in Rust; it
  does NOT aim to be a bash-compatible interpreter.
- **dash-rs / bash-rs**: No serious attempts at full POSIX sh / bash compatibility in Rust.

**Demand:** Moderate. Bash is everywhere in CI/CD. A faster, safer bash interpreter would be
useful for script-heavy environments. The GNU bash test suite would be the compatibility target.

---

## Community Discussions

### Reddit r/rust

Recurring thread patterns observed (2022–2025):

- **"RustPython: when will it support pip / NumPy?"** — Answered consistently: "It won't, no
  C extension support." Community is informed but periodically disappointed newcomers ask.
- **"Should I use mlua or rhai for scripting my game?"** — Active, practical discussions.
  Consensus: mlua if you want Lua ecosystem, rhai if you want something Rust-native and simpler.
- **"Boa vs QuickJS-rs vs rquickjs for embedding JS"** — Periodic threads. Consensus: rquickjs
  for production, Boa for research / contributing to a pure-Rust engine.
- **"YJIT rewrite to Rust — what does this mean?"** — Positive reception. Cited as proof that
  Rust can be used incrementally inside existing C runtimes.

### Hacker News

- **RustPython launch / milestones** (multiple HN posts, 200–500 points each): Comments focus
  on WASM demo praise and C-extension limitation disappointment.
- **"Deno 1.0" (2020, ~1000 points)**: HN discussion praised Rust's role in Deno's security
  model. Node community skeptical about compatibility.
- **"Wasmtime 1.0" (2022)**: Well-received. Discussions about WASI as a universal sandbox.
- **"Ruff — a Python linter 10–100x faster than flake8"** (2023, ~1200 points): Massive
  positive reception. Many comments: "This is what PyO3 + Rust enables." Sparked broader
  discussion about rewriting Python tooling in Rust.
- **"Should CPython be rewritten in Rust?"** — Periodic threads. Strong consensus: no, because
  the C extension ABI is load-bearing for the entire ecosystem. Incremental Rust (like YJIT's
  approach) is considered realistic; a full rewrite is not.

### The "Rewrite CPython in Rust" Discourse

This discussion recurs on HN and Reddit. The realistic analysis:

**Arguments for:**
- Memory safety — CPython has had many CVEs related to reference counting bugs, buffer overflows.
- Cleaner async story — CPython's asyncio is bolted on; a Rust rewrite could make async
  first-class.
- Better GC — CPython's refcounting + cycle collector is slow for certain patterns.

**Arguments against (dominant view):**
- The **CPython C extension ABI** (`Python.h`) is used by thousands of packages. Breaking it
  kills the ecosystem. PyPy's cpyext compatibility layer has been ~15 years of pain.
- Guido van Rossum and the CPython core team have explicitly said they are not interested in
  a Rust rewrite of the interpreter core.
- The GIL removal (PEP 703, shipped in Python 3.13 as experimental) is the more realistic
  improvement path.
- Cinder (Meta's CPython fork) and Faster CPython (the Microsoft-funded CPython optimization
  project) show the C codebase can be improved without rewriting.

**Realistic verdict:** A full CPython rewrite in Rust is not happening. Incremental Rust
integration (like YJIT in Ruby) in specific hot paths is plausible but not currently planned
by the CPython team.

---

## Failed Attempts

### Gluon (Rust ML language)

**Repo:** https://github.com/gluon-lang/gluon
**Status:** Abandoned / unmaintained (last significant activity 2021–2022).

Gluon was an ML-like statically typed functional language with a runtime written in Rust,
designed for embedding. It had type inference, a GC, and an async runtime. The project lost
momentum due to maintainer bandwidth. The language was interesting but never found significant
adoption. Lesson: building a new language runtime in Rust is tractable; finding users is harder.

### Mun Language

**Repo:** https://github.com/mun-lang/mun
**Status:** Stalled / very slow development.

Mun is a statically typed scripting language for game development with hot-reloading, written
in Rust. Technically impressive (LLVM-based AOT + hot reload). Development has significantly
slowed. Lesson: niche use cases (game scripting hot reload) need enough ecosystem momentum
(users, game engine integration) to sustain a runtime project.

### Ketos (Lisp in Rust)

**Repo:** https://github.com/murarth/ketos
**Status:** Abandoned (2016–2019).

Ketos was a Lisp scripting language for Rust. No GC improvements, no async, small user base.
Superseded by Steel, Rhai, and others.

### Wren-rs / Gravity-rs

Various attempts to port small scripting languages (Wren, Gravity) to Rust or provide Rust
bindings have stalled. The pattern: small team, pure hobby project, real-world embedding needs
win against pure reimplementation.

### Why Rust Runtime Projects Fail

Common patterns across failed/stalled projects:

1. **C-extension wall**: Any runtime trying to replace a language with a mature C extension
   ecosystem hits this hard and early.
2. **Single-maintainer bus factor**: Many Rust runtime projects are solo efforts. Key-person
   departure = project death.
3. **Performance regression**: "We wrote it in Rust so it should be fast" is not automatic.
   A naive AST-walking interpreter in Rust will be slower than CPython, let alone a JIT.
4. **Ecosystem gap**: A runtime without the language's package ecosystem is a toy. Building
   the runtime is 10 % of the work; the library ecosystem is 90 %.
5. **Undefined target market**: Projects that don't answer "who is this for?" struggle to
   attract contributors or users.

---

## Top Candidates (for Rust Runtime Projects Worth Building)

Ranked by: (impact × feasibility × unmet demand) — missing existing good solutions.

### Rank 1 — A Lua 5.4-compatible runtime with async/await and WASM support

**Why:**
- Lua is the dominant game scripting and embedded scripting language.
- Existing Rust options: mlua (C FFI, not pure Rust), Piccolo (promising but alpha).
- A production-quality pure-Rust Lua runtime that works in WASM and integrates with Tokio
  would be used immediately by game engines (Bevy scripting), web servers (Lapis-style), and
  IoT runtimes.
- Lua's standard library is small — no C-extension wall of the same magnitude as Python/Ruby.
- **Competitive moat:** No pure-Rust Lua runtime is production-ready. Piccolo is the closest.

**Estimated scope:** 1–2 dedicated engineers, 12–24 months to reach stable 1.0.

---

### Rank 2 — A fast, embeddable JavaScript engine in Rust (pure, no V8)

**Why:**
- Every server-side platform (Cloudflare Workers, Deno Deploy, Fastly) currently uses V8
  for JS isolation. V8 is large (binary size) and has high startup cost.
- Boa exists but is not production-ready and has no JIT.
- Nova has an interesting architecture but is pre-alpha.
- A production-grade ECMAScript engine in Rust with a JIT, small binary footprint, and WASM
  target would be directly useful for edge computing and plugin systems.
- **Competitive moat:** No pure-Rust JS engine is production-ready. This is a clear gap.

**Estimated scope:** 3–5 engineers, 2–4 years. High difficulty.

---

### Rank 3 — A Ruby runtime in Rust with YJIT-compatible output

**Why:**
- Ruby is slower than Python, and Python is considered slow. Rails performance is a constant pain.
- The community is receptive to Rust (YJIT precedent).
- A Rust Ruby runtime that targets MRI bytecode (YARV) compatibility — not reimplementing the
  parser from scratch — could provide a faster execution engine for existing Ruby code.
- **Competitive moat:** Artichoke is alpha; no production Ruby runtime in Rust exists.

**Estimated scope:** High difficulty. 3–5 years for serious compatibility.

---

### Rank 4 — A POSIX-compatible shell interpreter in Rust (bash replacement)

**Why:**
- CI/CD pipelines run billions of bash scripts per day.
- Bash is notoriously slow for loops, string operations, and subshell spawning.
- A POSIX-sh / bash-compatible Rust interpreter could provide significant speedups for
  script-heavy environments without requiring script rewrites.
- Security: bash has had many parsing-related CVEs (Shellshock, etc.).
- **Competitive moat:** No serious bash-compatible Rust interpreter exists.

**Estimated scope:** 1–3 engineers, 12–24 months for POSIX sh; another year for bash extensions.

---

### Rank 5 — PHP runtime in Rust targeting WordPress/Drupal workloads

**Why:**
- PHP's userbase is enormous and largely captive.
- A Rust PHP runtime with JIT that handled 90 % of typical WordPress code would reduce hosting
  costs for millions of sites.
- FrankenPHP (Go) showed demand for non-C PHP execution.
- **Competitive moat:** No PHP runtime in Rust exists at all.

**Estimated difficulty:** Extreme. PHP's type coercion semantics, superglobals, and extension
ecosystem are complex. But the core PHP opcodes are not more complex than Python's.

---

## Sources

- RustPython project: https://github.com/RustPython/RustPython
- RustPython playground: https://rustpython.github.io
- Deno project: https://github.com/denoland/deno
- Deno blog / architecture: https://deno.com/blog
- Boa JavaScript engine: https://github.com/boa-dev/boa
- Nova JS engine: https://github.com/trynova/nova
- mlua (Lua bindings for Rust): https://github.com/khvzalenko/mlua
- Piccolo (pure-Rust Lua): https://github.com/kyren/piccolo
- Artichoke Ruby: https://github.com/artichoke/artichoke
- Ruby YJIT (Rust): https://github.com/ruby/ruby/tree/master/yjit
- YJIT Rust rewrite announcement: https://shopify.engineering/yjit-faster-rubying-two-point-oh
- PyO3: https://github.com/PyO3/pyo3
- Wasmtime: https://github.com/bytecodealliance/wasmtime
- Wasmtime 1.0 announcement: https://bytecodealliance.org/articles/wasmtime-1-0
- Wasmer: https://github.com/wasmerio/wasmer
- WasmEdge: https://github.com/WasmEdge/WasmEdge
- Lunatic: https://github.com/lunatic-solutions/lunatic
- Fermyon Spin: https://github.com/fermyon/spin
- Rquickjs: https://github.com/DelSkayn/rquickjs
- rusty_v8: https://github.com/denoland/rusty_v8
- Rhai scripting: https://github.com/rhaiscript/rhai
- Steel Scheme: https://github.com/mattwparas/steel
- Apache DataFusion: https://github.com/apache/datafusion
- Gluon language: https://github.com/gluon-lang/gluon
- Mun language: https://github.com/mun-lang/mun
- NuShell: https://github.com/nushell/nushell
- Ruff linter: https://github.com/astral-sh/ruff
- CPython Faster CPython project: https://github.com/faster-cpython/ideas
- GraalVM Espresso: https://www.graalvm.org/latest/reference-manual/java-on-truffle/
- FrankenPHP: https://frankenphp.dev
- Polars DataFrame: https://github.com/pola-rs/polars
- Hacker News — RustPython threads: https://news.ycombinator.com/item?id=23123043
- Hacker News — Wasmtime 1.0: https://news.ycombinator.com/item?id=32873680
- Hacker News — Ruff: https://news.ycombinator.com/item?id=33494234
- Reddit r/rust — scripting embed discussions: https://www.reddit.com/r/rust/
- PEP 703 (No-GIL Python): https://peps.python.org/pep-0703/
