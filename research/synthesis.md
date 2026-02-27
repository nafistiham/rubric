# Research Synthesis: Top Rust Project Candidates

> **Synthesized from:** network-infra.md, dev-tooling.md, databases-data.md,
> language-runtimes.md, security-systems.md, scientific-ml.md
> **Note:** cli-tools.md was not available at synthesis time; findings from that domain
> were incorporated where they appeared in the other six files.
> **Date:** 2026-02-27

---

## Top 15 Candidates

### #1 Ruby Linter / Formatter (Rubocop Replacement)

- **What it replaces:** Rubocop (Ruby-based linter + formatter), standard-rb, pronto
- **Why Rust:** Pure speed. Rubocop is notorious for 8–15-minute CI lint runs on large Rails
  codebases. The Python ecosystem's transformation by Ruff is a direct proof-of-concept. The
  AST parsing problem is solved: `ruby-prism` (the new official Ruby parser) has Rust bindings,
  giving a correct, maintained AST without implementing a parser from scratch. Rust's parallel
  execution via Rayon provides the same structural speedup Ruff gets. No GC pauses during lint.
- **Evidence of demand:**
  - r/ruby has recurring threads where Rails shops report 8–15 min CI lint times.
  - rubocop GitHub issues tagged "performance" are among the most-starred issues in the repo.
  - Community explicitly asks for "a rubocop written in Rust like Ruff" — this phrasing appears
    verbatim in multiple Reddit threads (dev-tooling.md).
  - Enterprise Rails shops (Shopify, GitHub, Basecamp) are the primary sufferers; they have
    the engineering resources to adopt and contribute to a fast replacement.
- **Current gap:** No production-quality Rust Ruby linter exists. `ruby-prism` Rust bindings
  (https://github.com/ruby/prism) provide the parser; no one has built the lint engine on top.
- **MVP scope:** 4–8 weeks for a single engineer to produce a linter covering the top 50 most-used
  Rubocop cops (the ones that account for 80% of CI noise), with a formatter that matches
  standard-rb output, using ruby-prism for parsing. Ship as a cargo binary + a gem wrapper.
- **Risk:** Low-to-medium. The primary risk is cop coverage completeness — Rubocop has 400+
  cops. An MVP must prioritize ruthlessly. Secondary risk: Ruby version compatibility (prism
  handles Ruby 3.x; older syntax may need special handling).
- **Ranking rationale:** This is the single clearest "Ruff for X" opportunity that has not
  been taken. The demand is documented, the technical building blocks exist, the competitive
  landscape is empty, and the MVP scope is achievable by one developer. It follows a proven
  playbook. Ranked #1 because the effort-to-impact ratio is the highest of any candidate.

---

### #2 SQL Linter / Formatter (sqlfluff Replacement)

- **What it replaces:** sqlfluff (Python), sqlfmt, some use of black-compatible SQL formatters
- **Why Rust:** sqlfluff is "the slowest tool in our CI" — a direct quote that appears in
  multiple dbt community Slack threads. dbt projects can have hundreds of SQL models; sqlfluff's
  Python-based parser and rule engine makes it O(n) slow in an already slow language. Rust
  provides: fast parallel file processing, zero startup overhead, no GC pauses on large files.
- **Evidence of demand:**
  - sqlfluff GitHub issues tagged "performance" are among the highest-starred.
  - dbt community Slack is large (100k+ members); SQL linting speed is a recurring pain.
  - `sqruff` (https://github.com/quarylabs/sqruff) already exists — this is the strongest
    signal: someone already started building this because the demand was so obvious (dev-tooling.md).
  - Data engineering is a large, growing market with real CI costs ($$$).
- **Current gap:** sqruff exists but has limited dialect support and is early-stage. The gap
  is not "zero projects" but "no production-ready project." Opportunity to build on or significantly
  advance sqruff, or build a more complete competitor.
- **MVP scope:** 6–10 weeks for a binary that handles ANSI SQL + dbt's dialect (BigQuery, Snowflake,
  Redshift variants), with the top 20 most-used sqlfluff rules. Target dbt users specifically
  as the initial wedge market.
- **Risk:** Medium. SQL dialects are numerous and inconsistent. PostgreSQL SQL != MySQL SQL !=
  Snowflake SQL. Dialect coverage is the hard part. Leverage sqruff's existing parser work to
  reduce this risk.
- **Ranking rationale:** Documented community pain in a large, modern market (data engineering).
  A partial implementation already exists validating feasibility. Rust advantage is clear and
  measurable. MVP scope is realistic. The main risk (dialect coverage) is manageable with a
  focused initial scope.

---

### #3 sshd Drop-in (OpenSSH Server Replacement)

- **What it replaces:** OpenSSH's sshd (C, ~600k lines)
- **Why Rust:** This is the archetypal memory-safety argument. regreSSHion (CVE-2024-6387) was
  a remote unauthenticated root RCE on 14 million+ internet-facing servers — caused by a
  signal-handler race condition that is structurally impossible in safe Rust (no data races by
  design). The root cause was a C-specific memory safety class of bug. OpenSSH is on every Linux
  server. The impact of a production-ready Rust sshd is enormous.
- **Evidence of demand:**
  - CVE-2024-6387 generated explicit calls for a Rust sshd rewrite across HN, Reddit r/netsec,
    and security community forums (security-systems.md).
  - Prossimo (ISRG/Let's Encrypt's memory safety initiative) has funded sudo-rs, rustls, ntpd-rs,
    Hickory DNS — the logical next funded project in this sequence is sshd.
  - `russh` library (https://github.com/warp-terminal/russh) provides production-grade SSH
    server + client primitives; the gap is a complete, drop-in sshd binary, not the library.
  - Government mandates (White House ONCD Feb 2024, NSA, CISA) explicitly recommend Rust for
    exactly this class of problem.
- **Current gap:** russh is a library. No standalone sshd binary in Rust at production quality
  exists. This is directly comparable to the sudo-rs situation before Prossimo funded it.
- **MVP scope:** 8–16 weeks for an sshd binary that handles: public key auth, password auth,
  sftp subsystem, X11 forwarding, agent forwarding, basic sshd_config parsing. Built on top of
  russh. Pass a subset of the OpenSSH regression test suite.
- **Risk:** Medium-high. sshd_config compatibility is extensive. PAM integration is complex and
  platform-specific. The test surface is enormous (protocol versions, key types, client
  implementations). Security correctness is non-negotiable — a buggy Rust sshd is worse than
  no Rust sshd.
- **Ranking rationale:** Catastrophic CVE history creates urgent real demand. Library building
  blocks exist. Prossimo-funding potential is high. Security impact is among the highest of
  any Rust project. Ranked #3 rather than higher because scope and security correctness risk
  are significantly higher than #1 and #2.

---

### #4 Change Data Capture (CDC) Engine (Debezium Replacement)

- **What it replaces:** Debezium (Java/Kafka), Maxwell's Daemon (Java), DMS (AWS, proprietary)
- **Why Rust:** Debezium runs as a JVM process, often requiring 512MB–1GB RAM just for the
  connector framework. It needs ZooKeeper + Kafka as its output layer. A Rust CDC engine would:
  run with <50MB RAM, support direct Arrow Flight / Delta Lake / Iceberg outputs without
  requiring Kafka, and parse Postgres WAL / MySQL binlog safely (protocol parsing of untrusted
  binary data is exactly Rust's strength). The memory-safe protocol parsing argument is also
  a security argument — CDC connectors parse every database write; bugs here could corrupt
  downstream pipelines.
- **Evidence of demand:**
  - r/dataengineering threads consistently cite Debezium's JVM overhead as a pain point.
  - "Kafka requires 3 ZooKeeper nodes + 3 Kafka brokers for HA — insane for small teams" is
    a recurring complaint (databases-data.md).
  - The data engineering market is large and growing; every modern data stack uses CDC.
  - Delta Lake Rust (`delta-rs`) and Apache Iceberg Rust (`iceberg-rust`) exist as targets —
    a Rust CDC engine could output directly to these without a Kafka intermediary.
- **Current gap:** No Rust CDC engine with Postgres/MySQL logical replication support exists
  (databases-data.md). The Postgres logical replication protocol is documented. MySQL binlog
  parsing is more complex but well-understood.
- **MVP scope:** 8–12 weeks for a Rust binary that reads Postgres WAL via logical replication,
  decodes row changes, and outputs to: Kafka (for compatibility), Delta Lake (via delta-rs),
  and a simple JSON file/stream. Single-table CDC as the first target.
- **Risk:** Medium. Postgres WAL format changes between major versions. Handling schema changes
  (DDL events during CDC) is complex. MySQL binlog adds a second protocol surface.
- **Ranking rationale:** Strong market demand, no Rust competition, building blocks exist
  (delta-rs, iceberg-rust, tokio, pg protocol crates), and a clear MVP path. Appears in both
  databases-data.md and has security resonance from security-systems.md (safe protocol parsing).
  A cross-domain opportunity.

---

### #5 Config-Driven Reverse Proxy + Load Balancer (HAProxy / Caddy Replacement)

- **What it replaces:** HAProxy (C), Caddy (Go), nginx for proxy use cases
- **Why Rust:** Cloudflare's Pingora (Rust) handling 1 trillion requests/day with 70% CPU
  reduction and 67% memory reduction vs nginx is the landmark proof. The gap is not "can Rust
  do this" — it's proven. The gap is that Pingora is a *framework* requiring Rust code to
  configure, not an operator-friendly binary with a config file. HAProxy's configuration
  language is notoriously arcane. Caddy (Go) is user-friendly but memory-heavy. A TOML/YAML
  config-driven Rust reverse proxy + load balancer built on Pingora would fill the exact gap.
- **Evidence of demand:**
  - r/sysadmin and r/devops HAProxy configuration threads are consistently frustrated.
  - "HAProxy configuration is powerful but arcane" with "no native plugin model without Lua"
    is a documented, recurring pain (network-infra.md).
  - Every web-facing deployment needs a reverse proxy; this is one of the largest markets
    in infrastructure tooling.
  - Pingora's open-sourcing in 2024 was specifically to enable this kind of project.
- **Current gap:** No config-driven, operator-friendly Rust reverse proxy at production quality.
  Pingora is the building block; no one has built the "Caddy but in Rust" layer on top.
- **MVP scope:** 8–14 weeks for a single binary with TOML config supporting: HTTP/HTTPS
  reverse proxying, TLS termination via rustls, round-robin + weighted load balancing, health
  checks, basic rate limiting, and config hot-reload. Built on Pingora or Hyper.
- **Risk:** Medium. Operator adoption requires extreme stability — a reverse proxy that drops
  requests is worse than no reverse proxy. The long tail of HAProxy features (ACLs, stick
  tables, custom OCSP, etc.) makes "feature complete" a moving target.
- **Ranking rationale:** Very large market. Proven Rust performance story. Clear gap between
  Pingora-the-framework and a usable tool. Appears in network-infra.md as the #1 gap.
  Ranked #5 (not higher) because it requires more infrastructure expertise and has more
  competitive pressure (Traefik, Caddy, nginx are well-loved) than #1–#4.

---

### #6 Async Postgres Wire Protocol Server Library

- **What it replaces:** Custom per-project implementations, the incomplete `pgwire` crate
- **Why Rust:** Every Rust project that wants to be "Postgres-compatible" (RisingWave, Databend,
  Neon, custom query routers, PgBouncer alternatives) has had to independently implement the
  Postgres wire protocol in Rust. There is no single production-quality, Tokio-native async
  Postgres server protocol library. This is a high-leverage infrastructure library: one well-built
  crate would unblock a dozen other projects.
- **Evidence of demand:**
  - Multiple production Rust databases have independently reimplemented the Postgres wire
    protocol (databases-data.md). This duplication is the signal.
  - PgBouncer (C) is the dominant connection pooler; a Rust replacement using this library
    would be immediately buildable.
  - The `pgwire` crate exists but is described as "functional but limited."
- **Current gap:** No production-quality Tokio-based Postgres server protocol library. The
  client side (`tokio-postgres`, `sqlx`) is excellent; the server side is not.
- **MVP scope:** 4–6 weeks for a Tokio-based async library implementing: startup handshake,
  authentication (MD5, SCRAM-SHA-256), simple + extended query protocol, COPY protocol, and
  cancellation. Expose a trait-based API so users implement the query execution logic.
- **Risk:** Low-medium. The Postgres wire protocol is well-documented (PostgreSQL Frontend/
  Backend Protocol docs). The main risk is edge cases in the extended query protocol and
  handling client-library quirks (psql, pgAdmin, JDBC all have different behaviors).
- **Ranking rationale:** High leverage per effort invested. Library-level scope means MVP
  in weeks. Enables a dozen downstream projects. Clear documented gap. The risk/reward ratio
  is excellent. Ranked #6 because it's an enabling library rather than a user-facing product,
  limiting direct adoption visibility.

---

### #7 Authoritative DNS Server (BIND9 / PowerDNS Replacement)

- **What it replaces:** BIND9 (C), PowerDNS (C++)
- **Why Rust:** BIND has critical CVEs every single year — CVE-2023-3341, CVE-2023-4236,
  CVE-2024-1737 are all recent. DNS is critical internet infrastructure; memory bugs in DNS
  servers are catastrophic (DoS of authoritative servers affects entire domains). The memory
  safety argument is the strongest of any network infrastructure tool. Hickory DNS
  (https://github.com/hickory-dns/hickory-dns) exists but lacks: database backends for zone
  storage, a REST zone management API comparable to PowerDNS's, and production-scale testing.
- **Evidence of demand:**
  - "Ask HN: What infrastructure should be rewritten in Rust?" consistently cites DNS (BIND)
    as the top answer (network-infra.md).
  - ISRG/Prossimo is actively interested in this space; has already funded Hickory DNS work.
  - ISPs and registrars running BIND are directly exposed to its CVE history.
- **Current gap:** Hickory DNS has the primitives; nobody has built the production-grade
  authoritative server with PowerDNS-compatible API and database zone storage on top.
- **MVP scope:** 12–20 weeks for an authoritative-only DNS server (no recursive resolver)
  with: PostgreSQL zone storage, REST API for zone management, DNSSEC signing, and a
  configuration format familiar to BIND operators.
- **Risk:** High. Protocol completeness for DNS is extensive (dozens of RR types, DNSSEC,
  NSEC3, EDNS extensions). Production DNS infrastructure tolerates no errors. Niche audience
  (ISPs, registrars, self-hosters) limits initial adoption.
- **Ranking rationale:** Extremely strong security case. Documented community demand. Existing
  library foundation. Ranked #7 because scope and correctness requirements push MVP timeline
  to 4–5 months minimum and audience is more niche than #1–#6.

---

### #8 High-Performance ML DataLoader (Python Extension)

- **What it replaces:** `torch.utils.data` (Python/C++), NVIDIA DALI (CUDA/C++), WebDataset
- **Why Rust:** Data loading is frequently the GPU training bottleneck, not the GPU itself.
  Python's GIL prevents true parallel image decoding. Rust's ownership model + Rayon enables
  safe, parallel JPEG/PNG/TIFF decoding without GIL contention. PyO3 bindings mean Python
  users get a seamless `import dataloader_rs; dl = DataLoader(...)` experience — zero
  migration cost if the API matches `torch.utils.data`. Pattern proven by: Polars, HuggingFace
  tokenizers, pydantic-core — all Rust backends with Python frontends.
- **Evidence of demand:**
  - "Data loading is often the bottleneck" is a documented, recurring ML complaint
    (scientific-ml.md).
  - Python GIL as a data loading bottleneck is explicitly raised in r/MachineLearning threads.
  - NVIDIA DALI exists as evidence the problem is real, but DALI requires CUDA-specific
    infrastructure that doesn't work on CPU-only or MPS (Apple Silicon) systems.
  - ML teams at any scale (>10k samples) run into this.
- **Current gap:** No production Rust ML dataloader with PyO3 bindings matching
  `torch.utils.data`'s API. Burn has basic data loading; nothing Python-facing exists.
- **MVP scope:** 6–10 weeks for a Python library (via PyO3 + maturin) implementing: parallel
  JPEG/PNG decoding, basic augmentations (random crop, flip, normalize), batching, shuffling,
  WebDataset format support. API matches `torch.utils.data.DataLoader`.
- **Risk:** Low-medium. Image decoding libraries in Rust (`image`, `turbojpeg` bindings) are
  mature. PyO3 integration is well-understood. Main risk: matching the full `torch.utils.data`
  API surface including custom Dataset subclasses.
- **Ranking rationale:** Cross-domain (scientific-ml.md + dev-tooling.md pattern). Proven
  PyO3 pattern reduces technical risk. Large ML market. Clear bottleneck with measurable
  improvement. Ranked #8 because the market (ML practitioners) is Python-native and
  adoption requires PyTorch integration work.

---

### #9 IaC Security Scanner (Checkov / tfsec Replacement)

- **What it replaces:** Checkov (Python), tfsec (Go), trivy IaC scanning
- **Why Rust:** Checkov (Python) is notoriously slow on large Terraform repos — 5–15 minutes
  on a moderate-size infrastructure codebase is common. Security scanning is on the critical
  path of CI for compliance-driven organizations. A Rust-based HCL parser + rule engine would
  reduce scan times to seconds. tfsec (Go) is already faster than Checkov, but a Rust version
  could go further and add memory safety in the parser itself (HCL parsing of untrusted IaC
  files could theoretically be a vulnerability vector).
- **Evidence of demand:**
  - Checkov GitHub issues tagged "performance" are consistently starred.
  - DevSecOps teams at enterprise companies are the primary users; CI speed has direct cost
    impact (developer waiting time + runner costs).
  - IaC scanning is a compliance requirement in SOC 2, FedRAMP, and PCI-DSS environments —
    the market is mandated, not optional (dev-tooling.md).
- **Current gap:** No production Rust IaC scanner. `hcl2` parsing crates exist in Rust.
  The rule engine is a separate concern but well-understood (pattern matching on AST nodes).
- **MVP scope:** 6–10 weeks for a binary that parses HCL (Terraform) files and evaluates the
  top 100 CIS Benchmark rules for AWS. Output in SARIF format (GitHub integrations) and
  human-readable terminal output.
- **Risk:** Low-medium. HCL parsing is the main technical challenge; the rule engine is
  conceptually simple (AST traversal + pattern matching). Main risk: rule coverage completeness
  vs. Checkov's 1000+ rules.
- **Ranking rationale:** Mandated market (compliance), documented performance pain, no Rust
  competition, feasible MVP. Ranked #9 because tfsec (Go) already partially fills the speed
  gap, reducing the urgency of Rust vs. Go differentiation.

---

### #10 Network Packet Capture / Dissection Library (libpcap Replacement)

- **What it replaces:** libpcap (C), gopacket (Go)
- **Why Rust:** tcpdump, Wireshark, Suricata, and virtually every network security tool is
  built on libpcap. tcpdump has had multiple memory-corruption CVEs (CVE-2018-16301,
  CVE-2019-15166, etc.) — every single one caused by unsafe parsing of untrusted packet
  data. This is the definition of "where Rust should be." A production-quality, pure-Rust
  packet dissection library (handling Ethernet, IP, TCP, UDP, ICMP, DNS, HTTP, TLS at minimum)
  would unblock safe implementations of every network tool above it.
- **Evidence of demand:**
  - security-systems.md identifies tcpdump / libpcap replacement as Tier 1, #2 security gap.
  - pnet (https://github.com/libpnet/libpnet) exists as a low-level building block but lacks
    the higher-level protocol dissectors (no DNS parser, no TLS record parser, etc.).
  - "A safe packet dissection library would unlock many other tools" is the direct framing
    in security-systems.md.
- **Current gap:** pnet provides raw packet access; no Rust equivalent of gopacket's
  protocol-layer dissectors at production quality. Wireshark's C++ dissectors have 200+ CVEs.
- **MVP scope:** 8–12 weeks for a library with: BPF capture (via AF_PACKET or pcap crate FFI),
  Ethernet/IP/TCP/UDP/ICMP dissection, DNS packet parsing, TLS record parsing. Plus a tcpdump-
  like CLI binary built on top to demonstrate the library.
- **Risk:** Medium. Packet parsing correctness requires extensive testing against real captures.
  Platform-specific capture interfaces (Linux AF_PACKET vs macOS BPF vs Windows WinPcap) add
  complexity. Hardware offload (DPDK) is out of MVP scope.
- **Ranking rationale:** Extremely strong security case. Library-level scope enables an entire
  ecosystem. Clear gap. Ranked #10 (not higher) because the audience is primarily security
  professionals and network tool developers — a narrower initial adoption path than general
  developer tooling.

---

### #11 Prometheus-Compatible Time-Series Database

- **What it replaces:** VictoriaMetrics (Go), Prometheus itself (Go) for storage
- **Why Rust:** VictoriaMetrics's success ($100M+ equivalent ARR) and Prometheus's scaling
  limitations demonstrate massive market demand. Rust advantages: no GC pauses (critical for
  latency-sensitive metrics ingestion), compact memory layout for time-series data, and
  predictable tail latencies. GreptimeDB (Rust) supports PromQL but is not a drop-in Prometheus
  replacement. A Rust TSDB exposing the Prometheus remote_write/remote_read API would be
  immediately adoptable without any client changes.
- **Evidence of demand:**
  - VictoriaMetrics's growth validates the market for "faster Prometheus storage."
  - Prometheus scaling problems (single-node, high cardinality) are documented pain points.
  - GreptimeDB proves a Rust TSDB can be built (databases-data.md).
- **Current gap:** No drop-in Rust Prometheus-compatible TSDB. GreptimeDB is close but not
  positioned as a Prometheus replacement.
- **MVP scope:** 12–20 weeks for a single-node Rust TSDB with: remote_write ingestion API,
  remote_read query API, PromQL evaluation (using an existing PromQL parser crate), and WAL-
  based persistence. Compatible with Grafana out of the box.
- **Risk:** High. PromQL semantics are complex. Time-series compaction and efficient storage
  layout require deep expertise. Single-node only reduces scope but limits differentiation
  vs. Prometheus itself.
- **Ranking rationale:** Large market with validated demand. Rust advantages are concrete and
  measurable. Ranked #11 because the technical depth required is high and GreptimeDB already
  partially covers the space.

---

### #12 CSS Linter (stylelint Replacement)

- **What it replaces:** stylelint (Node.js-based)
- **Why Rust:** stylelint is Node.js-based and slow on large CSS codebases. LightningCSS
  (https://lightningcss.dev) already provides a production-quality, fast Rust CSS parser
  used inside Vite, Parcel, and Bun. Adding a lint rule engine on top of LightningCSS's
  existing AST is an incremental engineering effort. Following the exact Biome (lint +
  format in one binary) and Ruff patterns.
- **Evidence of demand:**
  - dev-tooling.md identifies this as a gap with "no production-quality stylelint replacement
    in Rust."
  - Every frontend project uses CSS; CI lint passes are a bottleneck.
  - stylelint has 10k+ GitHub stars and 2M+ weekly npm downloads — large user base.
- **Current gap:** LightningCSS handles parsing and transformation; no Rust lint rule engine
  on top. The building block (the parser) is already production-quality.
- **MVP scope:** 4–6 weeks for a binary implementing the top 30 most-used stylelint rules
  (covering 80% of typical .stylelintrc configs), with SARIF output and editor integration.
- **Risk:** Low. LightningCSS provides the parser. Rule logic is AST pattern matching.
  Main risk: rule coverage gaps vs. stylelint's 200+ rules causing adoption friction.
- **Ranking rationale:** Low-risk, clear gap, proven building block, large user base.
  Ranked #12 (not higher) because CSS linting is less painful than, e.g., Ruby linting —
  stylelint is slow but not 15 minutes slow on most codebases.

---

### #13 POSIX Shell Interpreter (Bash Replacement)

- **What it replaces:** GNU bash, dash, POSIX sh
- **Why Rust:** Bash is notoriously slow for loops and string operations. Shellshock
  (CVE-2014-6271) was a parsing-level bug in C that would not exist in safe Rust. CI/CD
  systems run billions of bash scripts; a faster, safer bash-compatible interpreter would
  reduce CI times for script-heavy pipelines. The security argument (untrusted script
  parsing) is real — build systems often process untrusted scripts.
- **Evidence of demand:**
  - language-runtimes.md identifies POSIX shell interpreter as Rank 4 runtime opportunity.
  - CI/CD is a documented pain point; bash slowness in loops is well-known.
  - No serious bash-compatible Rust interpreter exists (nsh is incomplete, nushell is a
    different paradigm).
- **Current gap:** No POSIX-sh-compatible Rust interpreter. The Go-based `mvdan/sh` library
  shows it can be done in a GC language; Rust should be able to do it with better performance.
- **MVP scope:** 10–16 weeks for POSIX sh compatibility (not full bash) covering: variable
  expansion, command substitution, pipes, conditionals, loops, functions. Pass the POSIX sh
  compliance tests. Full bash extension compatibility is a longer-term goal.
- **Risk:** Medium-high. POSIX sh is deceptively complex (IFS splitting, heredocs, quote
  handling). Real-world bash scripts use bash extensions; POSIX-only coverage is insufficient
  for many CI pipelines. Shell compatibility is a deep rabbit hole.
- **Ranking rationale:** Security + performance case is real. No Rust competition. Ranked
  #13 because POSIX-only coverage limits immediate usefulness and full bash compatibility
  is a multi-year project.

---

### #14 Embedded OLAP / Analytics Engine (DuckDB Alternative via DataFusion)

- **What it replaces:** DuckDB (C++), monetdb (C)
- **Why Rust:** DuckDB is the hottest database in data engineering (2023–2025). It is C++.
  DataFusion (Apache, pure Rust, production-grade) is the building block for a Rust DuckDB
  equivalent. GlareDB exists but is not a direct DuckDB replacement. A Rust embedded OLAP
  with: Python bindings (via PyO3), direct Parquet/Arrow/JSON/CSV file querying, and a SQL
  interface would attract the data science community — the same community that adopted Polars.
- **Evidence of demand:**
  - "When does DataFusion become DuckDB?" is the most common question in DataFusion HN threads
    (databases-data.md). The maintainers explicitly say: DataFusion is a library; someone
    needs to build the product.
  - DuckDB's star growth (70k+ stars) is among the fastest of any database tool in history.
  - Python data scientists who love DuckDB would instantly try a memory-safe, Rust-backed
    equivalent with PyO3 bindings.
- **Current gap:** DataFusion requires significant work to expose as a DuckDB-like product.
  GlareDB (https://github.com/GlareDB/glaredb) is the closest attempt but not a DuckDB
  clone.
- **MVP scope:** 10–16 weeks to produce a single binary + Python library that: reads
  Parquet/CSV/JSON from local disk and S3, executes SQL via DataFusion, and exports results
  as Arrow/Parquet/CSV. Wire up a DuckDB-compatible Python API surface.
- **Risk:** Medium. DataFusion's API is complex. DuckDB compatibility requires implementing
  DuckDB's SQL extensions. Performance competitive with DuckDB is not guaranteed — DuckDB
  has years of query optimization tuning.
- **Ranking rationale:** Proven building block (DataFusion). Large enthusiastic market.
  Clear gap documented by DataFusion maintainers themselves. Ranked #14 because DuckDB is
  free, excellent, and already has Python bindings — differentiation requires genuine effort.

---

### #15 PHP Linter / Static Analyzer (phpcs / Psalm Replacement in Rust)

- **What it replaces:** phpcs (PHP-based), php-cs-fixer (PHP-based), Psalm (PHP-based)
- **Why Rust:** PHP runs ~75% of the web (WordPress, Drupal, Laravel). PHP linting tools are
  PHP-based and slow. A Rust PHP linter following the Ruff/Biome playbook would provide
  10–100x speedup. The `php-parser` crate is incomplete but PHP's grammar, while complex,
  is well-documented.
- **Evidence of demand:**
  - dev-tooling.md identifies this as a gap with "both communities have active complaints."
  - Laravel and Symfony shops with large codebases face slow CI lint steps.
  - WordPress's dominance means the user base is enormous even if individual power users
    are limited.
- **Current gap:** No Rust PHP linter exists at any quality level. The parser is the main
  technical blocker.
- **MVP scope:** 12–18 weeks to build a PHP parser in Rust (or port an existing PHP ANTLR
  grammar to Rust via pest/nom) and implement the top 30 phpcs rules. Very slow MVP timeline
  due to parser complexity.
- **Risk:** High. PHP's grammar is genuinely complex (heredocs, nowdocs, type coercions,
  multiple incompatible language versions from PHP 7.4 to 8.3). Building a robust parser
  alone is a multi-month project. Ranked last because the parser risk is the highest of any
  linter candidate.
- **Ranking rationale:** Large market, no Rust competition, proven playbook — but held back
  by high parser complexity. Ranked #15 as the highest-potential but highest-risk linter
  opportunity.

---

## Cross-Domain Patterns

The following themes appeared consistently across three or more of the six research files:

### 1. The Ruff / Polars Playbook — Proven Template for Success

The single most repeated pattern across dev-tooling.md, scientific-ml.md, databases-data.md,
and security-systems.md is the "Ruff / Polars" model:

- Take a Python-ecosystem tool that is slow because it is written in Python
- Rewrite the core in Rust
- Expose a Python API via PyO3 that is identical or very similar to the original
- Ship as a native binary (for CLI tools) or a pip-installable package (for libraries)
- Watch adoption explode because the performance improvement is so large it overcomes
  any switching friction

This pattern is explicit in: Ruff (Flake8/Pylint replacement), uv (pip replacement), Polars
(Pandas replacement), pydantic-core (Pydantic acceleration), HuggingFace tokenizers, orjson.
The communities in Ruby (rubocop), SQL (sqlfluff), CSS (stylelint), PHP (phpcs), and scientific
computing (SciPy) are all explicitly asking for someone to apply this pattern to their domain.

### 2. Memory Safety as Security Argument — Strongest for Parsing Untrusted Data

Across network-infra.md, security-systems.md, databases-data.md, and language-runtimes.md:
the memory safety argument for Rust is strongest when the tool in question parses untrusted
input. This is not theoretical:

- OpenSSL (parses TLS data) → Heartbleed
- OpenSSH (parses SSH packets) → regreSSHion
- tcpdump / libpcap (parses network packets) → multiple CVEs
- BIND9 (parses DNS packets) → annual CVEs
- ClamAV (parses malware file formats) → heap overflows
- libarchive (parses ZIP/tar) → heap overflows

Every tool that processes untrusted binary data is a candidate for Rust replacement where
the security argument is concrete and the CVE history proves the point.

### 3. The "Framework vs. Tool" Gap — Libraries Without Products

Multiple research files identified the same structural problem: Rust has the library-level
building blocks but lacks the product-layer tooling built on top:

- Pingora (reverse proxy framework) → no config-driven HAProxy-like binary built on it
- Hickory DNS (DNS library) → no production authoritative server binary built on it
- russh (SSH library) → no sshd binary built on it
- DataFusion (query engine) → no DuckDB-like product built on it
- pnet (packet capture library) → no tcpdump-like tool built on it
- `pgwire` crate (Postgres protocol) → no production-quality library usable by others

This "last mile" gap is a recurring theme. The hard low-level work has been done; the
product-layer work (config files, CLIs, docs, operator UX) remains undone.

### 4. JVM / Python Ecosystem Lock-in — The Adoption Barrier

Across dev-tooling.md, databases-data.md, and language-runtimes.md: the biggest barrier to
Rust replacement adoption is not technical quality but ecosystem lock-in. Deno is a better
Node.js but couldn't displace npm. Fluvio is an interesting Kafka alternative but lacks
Kafka protocol compatibility. Artichoke is an interesting Ruby runtime but can't run Rails gems.

The projects that succeeded (Polars, Ruff, uv, boringtun) either: (a) exposed the original
API surface unchanged (PyO3 pattern), or (b) implemented full protocol compatibility (boringtun
implements the WireGuard protocol exactly). The projects that failed tried to replace both the
tool AND the ecosystem simultaneously.

### 5. Solo-Maintainer Bus Factor — The Organizational Risk

Across dev-tooling.md, scientific-ml.md, language-runtimes.md, and databases-data.md: project
abandonment is overwhelmingly caused by single-maintainer exhaustion, not by technical failure:

- Sled (KV store): Tyler Neely burnout
- Rome (JS toolchain): company funding collapse; Biome emerged from the community
- Gluon (ML language): maintainer stepped back
- Mun (game scripting): very slow development
- Gotham (web framework): maintainer stepped back
- juice (ML framework): maintainer exhaustion

Any new project should plan for maintainer succession and community building from day one.

### 6. The "Clean Slate" Temptation vs. "Build on Proven Primitives"

dev-tooling.md, databases-data.md, and security-systems.md all show the same pattern: projects
that tried to build everything from scratch (Sled rolling its own storage engine, juice building
GPU kernels from scratch, early Rust ML frameworks) stalled or failed. Projects that built on
proven primitives succeeded:

- Ruff builds on the Rust regex engine and AST crates
- Polars builds on Apache Arrow
- Pingora builds on Hyper and Tokio
- sudo-rs uses existing PAM C libraries via FFI for the parts that were already correct

The winning strategy: use Rust for the parts that matter (memory safety, performance, correct
business logic) and use proven C/C++ via FFI for the parts that are already trusted (BLAS,
OpenSSL where needed, established crypto implementations).

---

## Sweet Spot Projects

Projects that appear in multiple domains, have documented community evidence, have no serious
Rust competition, and are feasible to build:

### Sweet Spot #1: Ruby Linter (rubocop replacement)

- **Domains:** dev-tooling.md (Tier 1, #1 candidate)
- **Community evidence:** Explicit "please do what Ruff did for Python" requests on r/ruby;
  documented 8–15 min CI lint times in enterprise Rails shops
- **Rust competition:** None
- **Feasibility:** High — ruby-prism provides the parser; Ruff/Biome provide the template
- **Cross-domain bonus:** The Polars/Ruff playbook is proven; community knows this works

### Sweet Spot #2: SQL Linter (sqlfluff replacement)

- **Domains:** dev-tooling.md (Tier 1, #2 candidate); databases-data.md (touches dbt/SQL
  tooling ecosystem)
- **Community evidence:** dbt Slack, sqlfluff performance issues, sqruff's existence as
  market validation
- **Rust competition:** sqruff exists but is early-stage — opportunity to build on or
  compete with it
- **Feasibility:** High — sqruff validates the approach; focus on dbt dialect as wedge market

### Sweet Spot #3: sshd in Rust

- **Domains:** network-infra.md (security daemon), security-systems.md (Tier 1 gap #1)
- **Community evidence:** CVE-2024-6387 (regreSSHion) generated explicit public calls for
  a Rust rewrite; Prossimo funding model is available
- **Rust competition:** None (russh is a library; no sshd binary)
- **Feasibility:** Medium — russh provides the library; scope is large but bounded
- **Cross-domain bonus:** Government mandates (White House, NSA, CISA) create institutional
  demand independent of individual developer preference

### Sweet Spot #4: Config-Driven Reverse Proxy (built on Pingora)

- **Domains:** network-infra.md (#1 gap candidate); security-systems.md (TLS termination
  security argument)
- **Community evidence:** Documented operator frustration with HAProxy config, r/devops,
  r/sysadmin threads
- **Rust competition:** None (Pingora is a framework, not a tool)
- **Feasibility:** Medium — Pingora/Hyper/rustls provide all building blocks
- **Cross-domain bonus:** Every web-facing infrastructure deployment is a potential user

### Sweet Spot #5: ML DataLoader (Rust-backed Python library)

- **Domains:** scientific-ml.md (#2 Tier 1 candidate); dev-tooling.md (PyO3/maturin pattern)
- **Community evidence:** r/MachineLearning data loading bottleneck discussions; Python GIL
  complaint is universal in ML engineering
- **Rust competition:** None with PyO3 Python bindings
- **Feasibility:** High — PyO3 pattern is proven; image crates are mature; the API surface
  is well-defined (match torch.utils.data)
- **Cross-domain bonus:** Follows proven Polars/tokenizers playbook exactly

---

## Red Flags / Traps

### Trap 1: "Rewrite the CPython interpreter in Rust"

**Why it looks good:** Python is slow, Rust is fast, there are 19k GitHub stars on RustPython.

**Why it's a trap:** The C extension API (`Python.h`) is the entire Python ecosystem's
foundation. NumPy, pandas, scikit-learn — all are C extensions. RustPython explicitly
cannot run them and never will without implementing the C ABI, which is a decade-long project
that PyPy has spent 15+ years on and still hasn't completed. The community excitement (19k
stars) is driven by WASM demos, not production use. No production deployments. This is a
research project, not a product opportunity. (language-runtimes.md)

### Trap 2: "Build a Kafka-compatible broker in Rust"

**Why it looks good:** Redpanda ($100M+ ARR) proves the market. Kafka's JVM overhead is
a documented pain. Fluvio (Rust) exists.

**Why it's a trap:** Kafka protocol compatibility means implementing 50+ API versions with
exact byte-for-byte compatibility. The consumer offset management, partition leadership, ISR
tracking, and exactly-once semantics are each multi-month engineering efforts. Redpanda has
100+ engineers and years of head start. Fluvio abandoned Kafka compatibility and is still
not gaining traction. This is a years-long project, not a weeks-long MVP. The market
validation is real; the effort-to-reach-MVP ratio is prohibitive for a small team.
(databases-data.md)

### Trap 3: "Replace PostgreSQL"

**Why it looks good:** Postgres is C, Rust is safe, everyone uses Postgres.

**Why it's a trap:** PostgreSQL has 35+ years of development, a massive extension ecosystem,
millions of production deployments, and the trust that comes from decades of correctness
testing. SurrealDB and others have tried to position as "alternatives" and remain niche. The
"PostgreSQL replacement" is the canonical example of a project scope that would take decades
to achieve, not weeks to MVP. The correct play is extending PostgreSQL via pgrx, not replacing
it. (databases-data.md)

### Trap 4: "Build a full nmap replacement"

**Why it looks good:** nmap has CVEs, security professionals want it, RustScan shows appetite.

**Why it's a trap:** nmap's value is not port scanning — it's the Nmap Scripting Engine (NSE)
with 600+ scripts, OS fingerprinting with its database of 5000+ device fingerprints, and
service version detection with its probe database. Reimplementing these is each a multi-year
project. Multiple failed attempts exist on GitHub with <100 stars and no releases. Port
scanning (RustScan) is easy; the rest of nmap is not. (security-systems.md)

### Trap 5: "Build a Rust JVM"

**Why it looks good:** JVM cold start is a real pain (1–3 seconds on AWS Lambda), the
market is enormous, and Java/Kotlin/Scala are everywhere.

**Why it's a trap:** The JVM specification is enormous. JNI (native method interface) alone
is a multi-year project. A generational concurrent GC is a multi-year project. Class loading
with full reflective access is a multi-year project. No single company outside IBM, Oracle,
Eclipse (OpenJ9), and Azul has ever shipped a production JVM. GraalVM native-image is the
realistic answer and is already improving rapidly. A Rust JVM is a decade-long project, not
a feasible MVP. (language-runtimes.md)

### Trap 6: "Build the Rust GPU ML training framework (PyTorch killer)"

**Why it looks good:** PyTorch is Python/C++, Rust is better, Burn/Candle exist.

**Why it's a trap:** GPU kernel coverage is the moat. PyTorch has thousands of hand-optimized
CUDA kernels developed over a decade. Burn's WGPU backend is currently slower than PyTorch
for most workloads. Training large models requires distributed training (NCCL, pipeline
parallelism, tensor parallelism) — each is a multi-year engineering effort. The community
consensus from r/MachineLearning is: "Python isn't going away — we need Rust backends for
Python tools, not Rust replacements." Build the DataLoader, not the training framework.
(scientific-ml.md)

### Trap 7: "All-in-one toolchain" (the Rome mistake)

**Why it looks good:** Biome/Rome showed the vision; one tool for everything is appealing.

**Why it's a trap:** Rome Tools Inc. ran out of funding building an all-in-one JS toolchain.
The lesson: all-in-one tools are hard to fund, hard to market, and hard to maintain. Ruff
succeeded by being opinionated and narrow (a linter + formatter, not a bundler + compiler +
test runner). Biome succeeded by being the fork that focused on working features. Any new
project should start narrow, prove value in one domain, then expand. (dev-tooling.md)

### Trap 8: "Build a BGP routing daemon"

**Why it looks good:** FRRouting has CVEs, BGP is critical infrastructure, no Rust BGP daemon
exists — the gap is real.

**Why it's a trap:** BGP's state machine is complex, and real-world implementations from
major vendors (Cisco, Juniper, Arista) have quirks and bugs that any new implementation must
handle gracefully or face routing instability. The audience is ISPs and cloud providers —
organizations with the most extreme reliability requirements and the slowest adoption of new
software. The time to production adoption (even with a technically perfect implementation) is
measured in years. Niche audience + extreme correctness requirements + slow adoption = not a
feasible solo/small-team project for years. (network-infra.md)

---

## Recommendation

### The Single Best Project to Start: Ruby Linter / Formatter (Rubocop Replacement)

**Full Reasoning:**

The Ruff story is the most important data point in all six research files. In 2022, a single
engineer at Astral shipped Ruff — a Python linter written in Rust using the existing Python
AST ecosystem — and within 18 months it became the most-starred Python linter on GitHub,
adopted by Pydantic, FastAPI, Jupyter, and Airbnb. The key facts:

1. The demand was documented (Pylint takes 3 minutes; Ruff takes 2 seconds).
2. The parser was pre-existing (not built from scratch).
3. The API was compatible with the incumbent (same rule names, same config format).
4. Distribution was trivial (pip install ruff; cargo install ruff).
5. One person built the MVP; the community drove adoption.

The Ruby situation is identical in every structural dimension:

1. The demand is documented: 8–15 minute CI lint times on enterprise Rails codebases.
2. The parser is pre-existing: `ruby-prism` is Ruby's official new parser with Rust bindings
   (https://github.com/ruby/prism).
3. The API can be compatible: Rubocop's cops are well-specified; configuration format (.rubocop.yml)
   can be supported for common settings.
4. Distribution is straightforward: `gem install` wrapper + `cargo` binary.
5. The market is ready: r/ruby explicitly asks for this; the Shopify engineering team (who
   wrote YJIT in Rust!) is the ideal early adopter and potential contributor.

**No serious competition exists in Rust.** The field is empty. This is not true for most
of the other candidates, where partial implementations, related libraries, or competing
approaches already exist.

**The technical risk is the lowest of any impactful candidate.** ruby-prism provides a
correct, maintained Ruby parser. Ruff's open-source code provides a reference architecture.
The core engineering challenge is implementing lint rules against an AST — this is
algorithmically simple even if it is volume-intensive.

**The scope is achievable.** An MVP covering the 50 most-used Rubocop cops (which account
for the majority of CI lint violations) and a formatter matching standard-rb output can be
built by one engineer in 4–8 weeks of focused work. This is demonstrably faster than any
other top-10 candidate.

**The strategic position is excellent.** Ruby is #1's top domain that has not yet seen the
Ruff treatment. Python got Ruff, JavaScript got Biome/oxlint, Ruby has nothing. The community
awareness of the opportunity is high; the community will rally around a working implementation.
The Shopify engineering team's investment in Rust (YJIT) makes them natural early adopters
and potential contributors.

**Action plan:**
1. Week 1–2: Set up `ruby-prism` Rust bindings; build AST traversal infrastructure.
2. Week 3–4: Implement top 20 Rubocop cops (Style/*, Layout/* most common categories).
3. Week 5–6: Add formatter output matching standard-rb for the implemented rules.
4. Week 7–8: Ship a 0.1 binary; write a blog post with CI benchmark numbers.
5. Post-launch: Publish to r/ruby, r/rails, HN; let the community drive cop requests.

The risk-adjusted return on this project is higher than any other candidate in this research.

---

*End of Synthesis. Compiled from six domain research files covering:
network infrastructure, developer tooling, databases and data processing,
language runtimes, security systems, and scientific/ML computing.*
