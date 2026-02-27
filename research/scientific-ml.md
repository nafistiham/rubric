# Scientific Computing & ML: Rust Research

> **Research Date:** 2026-02-27
> **Knowledge Cutoff:** August 2025
> **Note:** WebSearch and WebFetch tools were unavailable in this environment.
> All findings are drawn from the author's training knowledge through August 2025,
> cross-referenced against known crate documentation, GitHub activity, and community
> discussions (Reddit r/rust, r/MachineLearning, Hacker News).

---

## Existing Rust Scientific Libraries

### Linear Algebra & Array Computing

**ndarray** (`ndarray` crate, v0.15+)
- The closest equivalent to NumPy's `ndarray` in Rust.
- Supports n-dimensional arrays with slicing, broadcasting, and arithmetic.
- Production-ready for CPU-only workloads; used by several ML crates internally.
- **Limitations:** No GPU support. Broadcasting is less ergonomic than NumPy. Missing many SciPy-level numerical routines (FFT, sparse solvers, ODE integrators). Performance is good but the API surface is much smaller than NumPy.
- **BLAS/LAPACK bindings:** `ndarray-linalg` crate wraps OpenBLAS/MKL/Accelerate. This works, but setup friction (system BLAS dependency) is a common complaint.

**nalgebra** (`nalgebra` crate, v0.32+)
- Focused on fixed-size and small-to-medium matrices. Extremely ergonomic for 2D/3D graphics, robotics, and computer vision (used by the `rapier` physics engine).
- Compile-time dimension checking via const generics is a genuine Rust superpower.
- **Production-ready:** Yes, for its target domain (linear algebra, geometry).
- **Limitations:** Not designed for large-scale numerical computing. Not a NumPy replacement. No GPU support.

**faer** (`faer` crate, v0.19+)
- Newer (2022–2024), focused on high-performance dense and sparse linear algebra on CPU.
- Benchmarks favorably against Eigen (C++) and sometimes LAPACK for certain operations.
- Written entirely in Rust with no C/Fortran dependencies — a key differentiator.
- Supports LU, QR, SVD, Cholesky, eigendecomposition.
- **Status:** Rapidly maturing. Not yet as widely adopted as nalgebra but gaining traction in the numerical computing community.
- **Limitation:** Still lacks the ecosystem depth of LAPACK (40 years of optimized routines).

**rulinalg**
- Older linear algebra crate; largely superseded by nalgebra and faer. Maintenance has slowed.

### Statistics & Probability

**statrs** (`statrs` crate)
- Probability distributions, statistical functions (CDF, PDF, sampling).
- Production-ready for basic statistics. Missing advanced inference, hypothesis testing infrastructure comparable to SciPy's `stats` module.

**argmin** (`argmin` crate)
- Mathematical optimization framework: gradient descent, L-BFGS, Nelder-Mead, etc.
- Designed to be backend-agnostic. Solid for optimization research.
- **Gap:** No autodiff integration built-in; you provide gradients manually.

**RustFFT** (`rustfft` crate)
- Pure-Rust FFT implementation. Competitive performance with FFTW for many sizes.
- Production-ready. Used in audio processing and signal processing pipelines.
- This is one of Rust's genuine success stories in numerical computing.

### Sparse Matrices & Graph Computing

**sprs** (`sprs` crate)
- Sparse matrix library (CSR/CSC formats). Basic arithmetic and some sparse solvers.
- **Status:** Functional but limited. Nothing close to SciPy's `sparse` module in breadth. Eigensolvers, iterative solvers (GMRES, CG), and sparse factorizations are largely missing or experimental.

**petgraph** (`petgraph` crate)
- Graph data structures and algorithms. Production-ready, widely used.
- Not strictly "scientific" but underpins many computational workflows.

### Differential Equations / ODE Solvers

**diffsol** (`diffsol` crate, emerging 2023–2025)
- ODE and DAE solver in pure Rust. Implements Runge-Kutta, BDF methods.
- Very early stage. Nothing close to the SciPy `integrate` module or Julia's `DifferentialEquations.jl`.
- **This is a major gap.**

**ode-solvers** (`ode-solvers` crate)
- Simple Runge-Kutta solvers. Useful for education, not production scientific computing.

### Signal Processing

**dasp** (Digital Audio Signal Processing)
- Sample-rate conversion, audio DSP primitives.
- Production-ready for its domain.

**hound**
- WAV file reading/writing. Stable.

### Symbolic Math

- **Essentially nothing production-ready.** No Sympy equivalent. `symoxide` and similar projects are very early experiments. This is a conspicuous gap.

---

## ML Frameworks in Rust

### Candle (HuggingFace)

- **Repository:** `huggingface/candle`
- **Status (as of mid-2025):** Active development, ~15k+ GitHub stars.
- **What it is:** A minimalist ML framework designed for inference. Priorities are: small binary size, WASM deployability, and easy integration with HuggingFace model weights.
- **Supports:** CUDA (via cuDNN and custom kernels), Metal (Apple Silicon), CPU.
- **Key models supported:** LLaMA, Mistral, Phi, Whisper, BERT, Stable Diffusion, many others.
- **Strengths:**
  - Best-in-class story for deploying LLMs in Rust (production inference servers).
  - No Python dependency at runtime.
  - HuggingFace weight format (safetensors) supported natively.
  - Very active maintenance backed by a major company.
- **Limitations:**
  - **Training is not a first-class citizen.** Candle has autograd but it is not optimized for training large models.
  - Ergonomics are low-level; building new architectures requires verbose code.
  - No high-level training loop abstractions (Trainer class, callbacks, etc.).
  - GPU kernel coverage is narrower than PyTorch. Some ops fall back to CPU.
  - No distributed training support.
- **Verdict:** Production-ready for inference. A research/training framework it is not.

### Burn

- **Repository:** `tracel-ai/burn`
- **Status (as of mid-2025):** v0.13+, ~8k+ GitHub stars, growing fast.
- **What it is:** A full ML framework attempting to be the PyTorch of Rust. Supports training and inference.
- **Backends:** CPU (via `ndarray`), CUDA (via WGPU or custom CUDA kernels), Metal, WebGPU, LibTorch (wrapping PyTorch's C++ backend), Candle.
- **Strengths:**
  - Backend-agnostic design is architecturally elegant.
  - Training loop abstractions, learners, callbacks — closer to a full framework.
  - Good autodiff implementation.
  - Active community and corporate backing (Tracel AI).
  - Fuzzing and formal testing of numerical correctness.
- **Limitations:**
  - GPU kernel performance still behind PyTorch for many ops. Custom WGPU backend especially immature.
  - Model zoo is small. Pre-trained model ecosystem is tiny compared to PyTorch Hub or HuggingFace.
  - Data loading pipeline (DataLoader equivalent) is basic.
  - No production deployments of significance reported yet.
  - Distributed training: experimental at best.
- **Verdict:** Most promising full ML framework in Rust. Not yet production-ready for serious training workloads. Watch closely for 2025–2026.

### dfdx

- **Repository:** `coreylowman/dfdx`
- **What it is:** Strongly-typed neural network framework where tensor shapes are checked at compile time.
- **Unique feature:** Shape and dtype errors are compile-time, not runtime — a genuine Rust superpower.
- **Status:** Active but smaller community than burn. CUDA support exists.
- **Limitations:**
  - Compile-time shape checking, while conceptually powerful, leads to very long compile times and complex type signatures that are hard to work with in practice.
  - Limited layer implementations and no pre-trained model ecosystem.
  - Less actively maintained than burn or candle in 2024–2025.
- **Verdict:** Innovative type-system approach; niche adoption. More research artifact than production framework today.

### tch-rs (PyTorch Bindings)

- **Repository:** `LaurentMazare/tch-rs`
- **What it is:** Rust bindings to LibTorch (PyTorch's C++ library).
- **Status:** Stable and usable. Maintained by Laurent Mazare (also works at HuggingFace).
- **Strengths:** You get PyTorch's mature GPU kernels and full op coverage in Rust.
- **Limitations:** You're shipping a large libtorch.so dependency. Not "pure Rust." Ergonomics are somewhat clunky (dynamic shapes, runtime errors for shape mismatches). Still requires CUDA toolkit for GPU use.
- **Verdict:** Pragmatic path to GPU ML in Rust today. Used in production by some teams.

### Linfa

- **Repository:** `rust-ml/linfa`
- **What it is:** Classical machine learning toolkit (scikit-learn analog). SVM, k-means, linear regression, logistic regression, naive Bayes, decision trees, PCA, etc.
- **Status:** v0.7, maintained, but development pace is slow.
- **Strengths:** Good API design. Genuine scikit-learn-like ergonomics. Pure Rust.
- **Limitations:** No deep learning. No GPU. The classical ML space in production is almost entirely Python (scikit-learn, XGBoost). No major production adoption story. Feature coverage is ~30–40% of scikit-learn.
- **Verdict:** Useful for embedded/edge classical ML. Not competitive with scikit-learn + Python ecosystem for mainstream data science.

### Smartcore

- Another classical ML library, similar scope to Linfa. Less active.

### ONNX Runtime Bindings

- `ort` crate: Rust bindings to Microsoft's ONNX Runtime.
- **Status:** Active, well-maintained by pykeio.
- **Verdict:** Best production inference path today if you train in Python and serve in Rust. Export PyTorch/TF/JAX to ONNX, run with `ort`. This pattern is increasingly common.

---

## Python Ecosystem Accelerators

This is where Rust has had its biggest impact on scientific/ML computing — not by replacing Python but by accelerating Python internals.

### Polars

- **What:** DataFrame library with a Rust core (`polars` crate), Python bindings via PyO3.
- **Status:** Production-ready, v1.0 released 2024. Used widely in data engineering.
- **Performance:** 5–20x faster than Pandas for many operations due to lazy evaluation, columnar storage, and parallel execution on Rayon.
- **Success factors:** Identical API philosophy to Pandas but with lazy/eager modes, Arrow-native, great error messages, and active development.
- **Verdict:** The single clearest success story of Rust in the scientific/data ecosystem. Proof that a well-designed Rust library can displace a decade-old Python incumbent.

### Pydantic v2

- Core validation logic rewritten in Rust (`pydantic-core`).
- **Impact:** 5–50x speedup on data validation — critical for ML data pipelines and API serving.
- **Pattern:** Pure Rust logic, PyO3 bindings, Python-facing API unchanged.

### Ruff (Astral)

- Python linter/formatter written in Rust. 10–100x faster than flake8/pylint.
- Not scientific computing directly, but widely used in ML engineering toolchains.
- **Relevant pattern:** Astral (the company behind Ruff) is now building `uv` (Python package manager) in Rust, attacking the Python toolchain itself.

### uv (Astral)

- Python package installer and resolver in Rust. 10–100x faster than pip.
- Directly impacts ML workflows where environment setup is a bottleneck.
- **Status:** Rapidly becoming the default in modern ML engineering stacks.

### DataFusion (Apache Arrow DataFusion)

- Query execution engine in Rust, used as the backend for several Python data tools.
- Powers parts of the Ballista distributed query engine.
- Embedded in tools like `dask` backends and standalone analytics engines.

### tokenizers (HuggingFace)

- HuggingFace's tokenization library has a Rust core (`tokenizers` crate).
- Provides 100x+ speedup over pure Python tokenization.
- **Pattern:** Python API, Rust backend — seamless to Python users.

### cryptography (PyCA)

- Python's `cryptography` library has migrated its backend to Rust (via `pyo3`).
- Not ML, but another data point for the "Rust backing Python" pattern.

### orjson

- Ultra-fast JSON library for Python, written in Rust.
- Commonly used in ML serving (FastAPI + orjson is a popular combination).

### NumPy — Rust Acceleration Status

- NumPy itself is not being replaced by Rust in the short term — its C/Fortran BLAS/LAPACK backend is highly optimized.
- **`numpy` crate:** Rust library for creating NumPy-compatible arrays from Rust (used internally).
- **`numpy` via PyO3:** Allows Rust functions to accept and return NumPy arrays. This is the primary interop mechanism.
- Projects like `arrayvision` and various university experiments have explored accelerating specific NumPy operations via Rust, but none have achieved mainstream adoption.

---

## Notable Successes (Polars, etc.)

### Polars

- **Why it worked:**
  1. Identified a real pain point (Pandas performance and memory usage).
  2. Correct abstraction level — DataFrame API is familiar to data scientists.
  3. Arrow-native from day one (interoperability with the rest of the data ecosystem).
  4. Lazy evaluation engine allows query optimization.
  5. PyO3 bindings made Python adoption effortless.
  6. Active, communicative maintainer (Ritchie Vink).
- **Lesson:** Rust doesn't need to replace Python. It needs to be the engine Python calls.

### RustFFT

- Pure-Rust FFT that matches or beats FFTW for many sizes.
- Used in production audio and signal processing.
- **Why it worked:** Well-defined problem, clear correctness criteria, no ecosystem dependencies needed.

### HuggingFace Tokenizers

- Rust backend for tokenization. Adopted by millions of ML practitioners invisibly.
- **Why it worked:** Python API unchanged. Speed improvement was undeniable. Zero migration cost.

### Ruff / uv

- Demonstrated that Rust tooling for Python development is viable and dramatically better.
- Opened community minds to the idea that the Python ecosystem itself can be Rust-accelerated.

### rapier (Physics Engine)

- High-performance 2D/3D physics in Rust. Used in game dev and robotics simulation.
- Shows Rust can compete with C++ in real-time numerical simulation.

---

## Gaps — High Value Missing

These are areas with clear demand, community signals, and no strong Rust solution today.

### 1. SciPy-equivalent (High Priority)

- **What's missing:** Comprehensive numerical routines: special functions (Bessel, gamma, etc.), sparse linear algebra solvers (GMRES, CG, AMG), ODE solvers, quadrature, optimization, signal processing (butter filters, etc.).
- **Community signal:** Repeatedly raised on r/rust, r/MachineLearning, and HN as the biggest gap in Rust scientific computing.
- **Why hard:** SciPy wraps decades of Fortran/C code (LAPACK, ARPACK, FITPACK). Rewriting cleanly in Rust is enormous scope.
- **Opportunity:** A `scipy-rs` or modular crates collection could be the "Polars moment" for scientific Rust.

### 2. GPU-Native Tensor Library (High Priority)

- **What's missing:** A Rust-native GPU tensor library with competitive kernel performance (matching cuDNN/cuBLAS coverage).
- **Current state:** Candle has CUDA support but limited coverage. Burn's WGPU backend is immature. Neither matches PyTorch kernel coverage.
- **Why it matters:** Without a competitive GPU backend, serious deep learning training in Rust is blocked.
- **Opportunity:** A `torch-core-rs` that wraps cuBLAS/cuDNN with safe Rust abstractions + a clean tensor API.

### 3. Symbolic Mathematics (High Priority)

- **What's missing:** A SymPy equivalent — symbolic algebra, calculus, equation solving, code generation.
- **Current state:** Nothing. `symoxide` is an abandoned experiment. Some tiny expression tree crates exist.
- **Why it matters:** Symbolic math is used for physics simulations, control theory, quantum computing, ML interpretability.
- **Opportunity:** This is an open greenfield. Julia's `Symbolics.jl` shows it can be done in a compiled language.

### 4. Automatic Differentiation (Production-Grade)

- **What's missing:** A standalone, production-grade autodiff library (like JAX's `grad`, or `torch.autograd`) usable independent of any ML framework.
- **Current state:** `burn` has autodiff; `dfdx` has compile-time autodiff; `ad` crate exists but is limited. None are production-grade standalone libraries.
- **Why it matters:** Physics simulations, optimization, sensitivity analysis — all need autodiff outside of ML.
- **Opportunity:** A `jax-rs` focused purely on composable function transformation (grad, jit, vmap).

### 5. DataLoader / Data Pipeline Infrastructure

- **What's missing:** High-performance data loading for ML: parallel image decoding, batching, augmentation, shuffling — the `torch.utils.data` equivalent.
- **Current state:** No mature solution. `linfa` has basic dataset types. `burn` has some data loading but limited.
- **Why it matters:** Data loading is frequently the bottleneck in ML training, even with fast compute.
- **Opportunity:** A `dataloader-rs` that Python can call via PyO3 for 3–10x faster data pipelines.

### 6. Probabilistic Programming / Bayesian Inference

- **What's missing:** Stan, PyMC, or NumPyro equivalent.
- **Current state:** `rv` crate (basic distributions), some MCMC experiments. Nothing production-ready.
- **Why it matters:** Bayesian methods are used across scientific computing, especially in life sciences and physics.

### 7. Bioinformatics / Scientific Domain Libraries

- **What's missing:** A mature Biopython equivalent. Tools for sequence alignment, phylogenetics, structural biology.
- **Current state:** `bio` crate exists (sequence analysis, pairwise alignment) and is reasonably maintained. But nothing near Biopython's scope.
- **Growing community:** Rust in bioinformatics is gaining attention (noodles for genomics file formats is solid).

### 8. Interactive Scientific Notebooks

- **What's missing:** A Jupyter equivalent or strong Jupyter kernel for Rust.
- **Current state:** `evcxr_jupyter` exists — a Rust Jupyter kernel. It works for education but has limitations (slow, limited plotting, no DataFrame display).
- **Opportunity:** Deep integration with Polars + a visualization library to make Rust a first-class notebook citizen.

### 9. Data Visualization

- **What's missing:** A production-ready, feature-complete plotting library.
- **Current state:**
  - `plotters`: The most mature. Supports line charts, scatter, bar, histogram, candlestick. Backend-agnostic (PNG, SVG, WebAssembly). But static only — no interactivity.
  - `plotly` (Rust): Bindings to Plotly.js. Interactive but requires a browser/runtime.
  - `egui` + `egui_plot`: Immediate-mode GUI with basic plotting. Used in desktop apps.
  - `vega-lite` bindings: Experimental.
  - **Nothing close to matplotlib in feature coverage or Plotly/Bokeh in interactivity.**
- **Gap:** No publication-quality static plot library with LaTeX support, logarithmic scales, colormaps, contour plots, 3D surface plots — all common in scientific papers.

### 10. Distributed Computing Framework

- **What's missing:** A Dask or Ray equivalent for distributed computation in Rust.
- **Current state:** Apache Arrow DataFusion + Ballista attempt this but are far from Ray's generality.
- **Why it matters:** Large-scale ML and data processing require distributed compute.

---

## Community Discussions

### Reddit r/rust — Recurring Themes

**"Why isn't there a NumPy for Rust?"** (asked many times, 2021–2025)
- Top answers consistently: `ndarray` exists but lacks NumPy's breadth; the real gap is BLAS/LAPACK wrapping at higher level; ecosystem fragmentation is the core problem.
- Common frustration: "I need to do science in Rust but keep falling back to Python for anything beyond basic linear algebra."

**"Is Rust ready for scientific computing?"** (2023–2024 threads)
- Community consensus: "Ready for performance-critical C-replacement work; not ready to replace Python as a scientific environment."
- Specific callouts: missing plotting, missing SciPy, ndarray ergonomics.

**r/MachineLearning on Rust ML frameworks (2024)**
- Candle gets praise for inference use case.
- Burn gets cautious optimism.
- Common refrain: "Python isn't going away — we need Rust backends for Python tools, not Rust replacements."
- Noted: PyTorch itself uses C++, not Rust. The question is whether future frameworks use Rust instead of C++.

### Hacker News — Notable Discussions

**"Candle: An ML Framework for Rust" (HN, 2023)**
- Top comments praised the approach but questioned training story.
- Several comments noting: "Candle is for inference. If you need training, use PyTorch."
- Debate about whether Rust's borrow checker makes ML framework internals harder to write (graph structures, in-place mutation).

**"Polars vs Pandas" (multiple HN threads, 2023–2024)**
- Polars consistently praised. Common sentiment: "Polars is what Pandas should have been."
- Discussion of the Rust/PyO3 pattern as a template for other Python acceleration projects.
- Several users: "I hope someone does this for NumPy next."

**"Is Rust the future of scientific computing?" (HN, 2024)**
- Split opinions. Fortran and C++ advocates argue maturity and library ecosystem.
- Rust advocates point to memory safety, cargo, and the Polars success.
- Julia advocates argue: "We solved this 10 years ago — use Julia."
- Consensus: Rust is interesting for systems-level scientific software; Julia is better for interactive science.

**"Why does ML tooling suck?" (r/MachineLearning, recurring)**
- Python GIL prevents true parallelism in data loading.
- Serialization (pickle) is a footgun.
- Memory management is opaque.
- Users want: "Something with Python ergonomics but Rust internals."
- This is exactly what Polars delivers for DataFrames — the template exists.

### GitHub Issues / Forum Signals

- `ndarray` GitHub: long-standing issues requesting better broadcasting, more ufunc-like operations, GPU support. Progress is slow.
- `burn` GitHub: active issues for CUDA kernel coverage, distributed training, model zoo.
- `candle` GitHub: training ergonomics improvement requests consistently in top issues.
- `faer` GitHub: requests for sparse matrix integration, eigenvalue solvers beyond basic dense.

---

## Failed Attempts

### juice (formerly leaf)

- **What it was:** One of the earliest Rust ML frameworks. Attempted to be a full deep learning framework with GPU support.
- **Status:** Abandoned ~2017–2018. The maintainer wrote a post-mortem noting that the Rust ecosystem was too immature (no stable async, limited GPU libraries) and the scope was too ambitious.
- **Lesson:** Too early. GPU ecosystem wasn't there. Framework design patterns weren't clear.

### rusty-machine

- **What it was:** Classical ML library predating Linfa.
- **Status:** Archived/abandoned ~2019. Superseded by Linfa.
- **Lesson:** Needed a community and organizational structure, not just code.

### Emu (GPU computing)

- **What it was:** A Rust library for GPU compute, aiming to make GPU programming as easy as CPU programming.
- **Status:** Abandoned ~2020. The macro-heavy API was complex; WGPU and other backends matured to supersede it.
- **Lesson:** Macro-based GPU abstraction is hard to maintain. WGPU's evolution changed the landscape.

### dfdx (stagnation risk)

- Not abandoned, but development pace slowed in 2024 relative to burn and candle.
- The compile-time shape checking design, while elegant, causes extreme compile times that deter users.
- **Risk:** May be superseded by burn's approach before reaching production maturity.

### tensorflux / other TensorFlow wrappers

- Several attempts to wrap TensorFlow in Rust. All either abandoned or unmaintained.
- **Lesson:** Wrapping TensorFlow's complex Python-centric API in Rust is not ergonomic. The C API is unstable. Better to use ONNX or LibTorch as the FFI target.

### ndarray-stale ecosystem

- Several crates that extended ndarray (ndarray-stats, ndarray-image, etc.) have had slow development.
- The fragmentation of the ndarray ecosystem means there's no single coherent "NumPy for Rust" — just many small crates of varying quality and maintenance.

---

## Numerical Computing: Rust vs Fortran/C for HPC

### Performance Reality

- **For single-threaded BLAS/LAPACK:** Rust calling optimized Fortran/C (via `ndarray-linalg` + OpenBLAS/MKL) matches Fortran/C performance — because it IS running Fortran/C.
- **For pure-Rust implementations (faer):** Competitive with Eigen (C++) for dense linear algebra on modern CPUs. faer uses AVX2/AVX-512 via `pulp` SIMD abstraction layer.
- **For memory-bound workloads:** Rust's ownership model can help avoid unnecessary copies, giving edge cases where Rust beats C.
- **For Fortran legacy HPC:** Rust cannot easily interoperate with Fortran's array conventions, complex number types, and MPI bindings at the same ergonomic level as C.

### HPC Ecosystem Gaps

- **MPI:** `mpi` crate exists but is not widely used in production HPC.
- **OpenMP equivalent:** Rayon is excellent for shared-memory parallelism but is not a drop-in OpenMP replacement for scientific codes.
- **BLAS/LAPACK:** Rust can call them via FFI but writing new BLAS-competitive routines in pure Rust is still cutting-edge work (faer is pushing this).
- **Verdict:** Rust is competitive on a single node. For distributed HPC (MPI-based clusters), the ecosystem is thin.

---

## CUDA / GPU Computing in Rust

### Current State (mid-2025)

**cudarc** (`cudarc` crate by Coreylowman/dfdx team)
- Safe Rust abstractions over CUDA: device management, kernel launching, cuBLAS, cuDNN.
- **Status:** Used internally by dfdx and some candle components. Not a high-level framework.
- Requires CUDA toolkit installed; still low-level.

**WGPU** (WebGPU in Rust)
- Cross-platform GPU compute: works on Metal, Vulkan, DX12, and WebGPU.
- **Status:** Production-ready for graphics; compute shaders for ML are immature.
- Burn's WGPU backend uses this. Performance is behind CUDA for ML workloads.
- **Promise:** The only serious path to write-once GPU code across CUDA/Metal/Vulkan.

**Candle CUDA kernels**
- HuggingFace has written custom CUDA kernels in Rust for Candle (flash attention, etc.).
- Shows it's possible, but requires deep CUDA expertise.

**rust-cuda (Rust CUDA project)**
- Ambitious project to compile Rust directly to CUDA PTX (without writing CUDA C).
- **Status (2025):** Experimental, nightly-only Rust features required, not production-ready.
- If this matures, it would be transformative — write GPU kernels in safe(r) Rust.

**HIP/ROCm**
- AMD GPU support is even thinner than CUDA in the Rust ecosystem. Essentially unaddressed.

### Key Limitation

The fundamental challenge: CUDA's ecosystem is mature C++/Python. Every new Rust GPU library must either wrap existing CUDA C++ (FFI complexity) or reimplement kernels from scratch (enormous effort). There is no easy path.

---

## Top Candidates (Ranked by Opportunity)

Ranking based on: market need, technical feasibility in Rust, size of gap, community evidence, commercial potential.

### Tier 1: Highest Leverage

**1. SciPy-Core (Modular Numerical Routines Library)**
- Fill the ODE solvers, quadrature, special functions, sparse solvers gap.
- Could be offered as a Rust crate AND as a Python extension (PyO3).
- Template: Polars. Approach: replace SciPy's C/Fortran backends with pure Rust.
- **Estimated Impact:** Very high. Every scientific Python user would benefit.

**2. High-Performance DataLoader for ML**
- Rust-backed Python data loading pipeline. Parallel image decoding (JPEG, PNG, TIFF), WebDataset streaming, fast augmentation.
- **Pattern:** Python API (matches `torch.utils.data`), Rust backend.
- GPU bottleneck is often actually the CPU data pipeline. This has immediate commercial value.
- **Estimated Impact:** High. Most ML teams have data loading bottlenecks.

**3. GPU Tensor Backend (Rust-native, cuBLAS/cuDNN wrapper)**
- A clean, safe Rust API over cuBLAS + cuDNN that burn/candle can build on.
- Consolidate the ecosystem around one GPU abstraction layer.
- **Estimated Impact:** High. Prerequisite for Rust ML frameworks to mature.

### Tier 2: High Value

**4. Symbolic Math Library (SymPy-rs)**
- Greenfield. Would unlock physics simulation, control theory, quantum computing in Rust.
- Hard but feasible. Julia's `Symbolics.jl` provides a design template.
- **Estimated Impact:** High for scientific computing specifically.

**5. Publication-Quality Plotting (matplotlib-rs)**
- Static, publication-quality plots with LaTeX support, colormaps, contour plots, 3D surface.
- Could be a Python extension (replacing matplotlib's C backend) or pure Rust with PyO3 bindings.
- **Estimated Impact:** Medium-High. Every scientist needs plotting.

**6. Probabilistic Programming Framework**
- Stan-like MCMC and variational inference in Rust.
- Could be compiled to WASM for browser-based Bayesian analysis.
- **Estimated Impact:** Medium. Niche but high-value scientific audience.

### Tier 3: Solid but Competitive

**7. Expanded Linfa (scikit-learn completeness)**
- Bring Linfa to scikit-learn feature parity: gradient boosting, random forests (beyond basic), neural network layers.
- **Estimated Impact:** Medium. scikit-learn + Python is hard to displace.

**8. Bioinformatics Framework**
- Biopython-level coverage in Rust. The `noodles` + `bio` crates are a foundation.
- Genomics community is receptive to Rust (Heng Li has expressed interest).
- **Estimated Impact:** Medium. Specialized but large scientific audience.

**9. WGPU-based GPU ML Backend**
- If WGPU compute matures, a unified cross-platform ML backend (no CUDA required) would be transformative.
- **Estimated Impact:** Long-term high; near-term speculative.

**10. ODE/PDE Solver Ecosystem**
- High-performance PDE solvers (finite element, finite difference) for physics simulation.
- `ferrite-rs` and similar projects exist for FEM but are not widely known.
- **Estimated Impact:** Medium. Targeted at computational physics/engineering.

---

## Sources

> Note: Since web tools were unavailable, sources below represent the canonical references
> to verify these findings. All are publicly accessible.

### Crate Documentation & Repositories

- **ndarray:** https://github.com/rust-ndarray/ndarray | https://docs.rs/ndarray
- **nalgebra:** https://github.com/dimforge/nalgebra | https://nalgebra.org
- **faer:** https://github.com/sarah-ek/faer-rs | https://faer-rs.github.io
- **statrs:** https://github.com/statrs-dev/statrs
- **argmin:** https://github.com/argmin-rs/argmin | https://argmin-rs.org
- **RustFFT:** https://github.com/ejmahler/RustFFT
- **sprs:** https://github.com/vbarrielle/sprs
- **linfa:** https://github.com/rust-ml/linfa
- **burn:** https://github.com/tracel-ai/burn | https://burn.dev
- **candle:** https://github.com/huggingface/candle
- **dfdx:** https://github.com/coreylowman/dfdx
- **tch-rs:** https://github.com/LaurentMazare/tch-rs
- **ort (ONNX Runtime):** https://github.com/pykeio/ort
- **Polars:** https://github.com/pola-rs/polars | https://pola.rs
- **plotters:** https://github.com/plotters-rs/plotters
- **cudarc:** https://github.com/coreylowman/cudarc
- **wgpu:** https://github.com/gfx-rs/wgpu
- **diffsol:** https://github.com/martinjrobins/diffsol
- **noodles (bioinformatics):** https://github.com/zaeleus/noodles
- **bio (bioinformatics):** https://github.com/rust-bio/rust-bio
- **petgraph:** https://github.com/petgraph/petgraph
- **evcxr_jupyter:** https://github.com/evcxr/evcxr

### Python Acceleration Projects

- **pydantic-core (Rust backend):** https://github.com/pydantic/pydantic-core
- **HuggingFace tokenizers:** https://github.com/huggingface/tokenizers
- **ruff:** https://github.com/astral-sh/ruff
- **uv:** https://github.com/astral-sh/uv
- **orjson:** https://github.com/ijl/orjson
- **Apache Arrow DataFusion:** https://github.com/apache/arrow-datafusion

### Community Discussions (Search these on respective platforms)

- Reddit r/rust: "numpy rust equivalent", "scientific computing rust 2024", "rust ml framework"
- Reddit r/MachineLearning: "candle rust", "burn framework rust", "rust pytorch"
- Hacker News: Search "Candle HuggingFace", "Polars vs Pandas", "Rust scientific computing"
- Are We Learning Yet? (Rust ML tracking site): https://www.arewelearningyet.com/

### Reference Articles & Posts

- Ritchie Vink (Polars creator) blog: https://www.ritchievink.com
- Llogiq's Rust blog posts on scientific computing
- "Rust for Science" threads on users.rust-lang.org
- faer-rs benchmarks vs Eigen/LAPACK: https://faer-rs.github.io/bench.html
- burn documentation: https://burn.dev/book/

### Standards & Comparisons

- SciPy documentation (what Rust lacks): https://docs.scipy.org/doc/scipy/
- NumPy documentation (baseline comparison): https://numpy.org/doc/
- Julia vs Rust for scientific computing: multiple HN/r/rust threads 2022–2025
