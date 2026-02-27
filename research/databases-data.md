# Databases & Data Processing: Rust Research

> **Note on sourcing:** WebSearch and WebFetch tools are unavailable in this environment.
> All claims below are drawn from the author's knowledge base (cutoff August 2025),
> with primary sources cited by URL for independent verification. Dates are noted
> where recency matters.

---

## Existing Rust Databases & Storage

### SurrealDB
- **What it is:** Multi-model database (document, graph, relational, key-value) written entirely in Rust.
- **Storage backends:** Pluggable — RocksDB (on-disk), TiKV (distributed), in-memory, or IndexedDB (WASM).
- **Query language:** SurrealQL — SQL-like with graph traversal, live queries, and embedded scripting via WebAssembly.
- **Production status:** v1.0 released September 2023; v2.0 released 2024. Used in production by a growing number of startups. Cloud offering (Surreal Cloud) in beta as of mid-2025.
- **Strengths:** True multi-tenancy, schema-optional, WebSocket live queries, built-in auth and permissions layer, WASM embeddability. One of the most starred Rust database projects (~27k+ stars).
- **Weaknesses:** Still maturing under high-throughput write scenarios; limited ecosystem tooling compared to Postgres.
- **Source:** https://github.com/surrealdb/surrealdb | https://surrealdb.com/docs

### TiKV
- **What it is:** Distributed transactional key-value store, written in Rust. Originally the storage layer for TiDB (a distributed NewSQL database).
- **Production status:** CNCF graduated project. Running in production at scale at companies like Shopify, Xiaomi, Zhihu. Handles petabytes of data.
- **Key design:** Raft consensus, MVCC, pessimistic and optimistic transactions, Region-based sharding.
- **Strengths:** Battle-hardened distributed transactions, excellent horizontal scalability, widely deployed.
- **Weaknesses:** Complex operational overhead; not designed as a standalone product — usually consumed via TiDB or the raw client.
- **Source:** https://github.com/tikv/tikv | https://tikv.org

### Neon (Serverless Postgres)
- **What it is:** Not a Rust database per se, but Neon's storage engine and compute separation layer are written in Rust. It reimplements the Postgres storage layer as a multi-tenant, copy-on-write page server.
- **Production status:** GA as of 2024. Venture-backed ($46M Series B). Used by thousands of developers.
- **Rust components:** `pageserver` (the remote storage layer), `safekeeper` (WAL service), and `proxy`. The Postgres compute nodes themselves remain C.
- **Strengths:** Branching (instant database copies), serverless scale-to-zero, separation of compute and storage — all enabled by Rust's memory safety in the critical storage path.
- **Source:** https://github.com/neondatabase/neon | https://neon.tech/blog/architecture

### CeresDB / HoraeDB
- **What it is:** Cloud-native time-series and analytical database from Ant Group, written in Rust. Renamed to HoraeDB in 2023 after open-sourcing.
- **Production status:** Used internally at Ant Group at scale; community adoption is growing but limited outside China.
- **Key features:** Hybrid row-column storage, time-series optimized compaction, SQL support via DataFusion.
- **Source:** https://github.com/apache/horaedb

### Databend
- **What it is:** Cloud-native data warehouse (OLAP) written in Rust. Positioned as an open-source Snowflake alternative.
- **Production status:** v1.0 in 2024. Has a managed cloud offering. Growing enterprise adoption.
- **Strengths:** Object storage native (S3/Azure/GCS), vectorized execution, Apache Arrow in-memory format, DataFusion query engine integration, built-in semi-structured data support (JSON, Parquet, CSV).
- **Weakness:** Ecosystem maturity vs. Snowflake/BigQuery; not yet widely adopted outside cost-sensitive use cases.
- **Source:** https://github.com/datafuselabs/databend | https://databend.com

### GlueSQL
- **What it is:** Rust SQL database engine designed to be embedded anywhere — file systems, in-memory, browser (WASM), custom storage backends.
- **Production status:** Active development, used in some edge/embedded contexts. Not as mature as SQLite.
- **Source:** https://github.com/gluesql/gluesql

### Qdrant
- **What it is:** Vector similarity search database written in Rust.
- **Production status:** One of the most production-ready vector databases available. Used in production at many AI companies. Reached v1.0 in 2023, v1.9+ in 2025.
- **Strengths:** High performance HNSW indexing, filtered search, scalar/product quantization, rich payload filtering, gRPC + REST APIs, built-in distributed mode.
- **Source:** https://github.com/qdrant/qdrant | https://qdrant.tech

### LanceDB
- **What it is:** Serverless vector database built on the Lance columnar format, written in Rust.
- **Production status:** Active development, embedded-first design (similar to DuckDB's philosophy but for vectors). Growing adoption in AI/ML pipelines.
- **Strengths:** Works on local disk, S3, or GCS without a server; native Apache Arrow; multimodal data support.
- **Source:** https://github.com/lancedb/lancedb

### GreptimeDB
- **What it is:** Distributed time-series database written in Rust, designed for cloud-native deployments.
- **Production status:** v0.9+ in 2025, gaining traction especially in IoT and observability.
- **Key features:** SQL + PromQL support, object storage backend, columnar storage for time-series, compatible with InfluxDB line protocol.
- **Source:** https://github.com/GreptimeTeam/greptimedb

### RisingWave
- **What it is:** Distributed SQL streaming database written in Rust — processes real-time streams with the full power of SQL.
- **Production status:** v1.0 in 2023, v2.x in 2025. Production deployments at multiple companies.
- **Key features:** Materialized views that update in real-time, Postgres-compatible SQL, Kafka/Kinesis/Pulsar sources, OLTP-grade consistency.
- **Source:** https://github.com/risingwavelabs/risingwave

### Redb
- **What it is:** Embedded key-value database written in pure Rust. Designed as a safer, simpler alternative to LMDB.
- **Production status:** v2.x in 2024. Used as a storage backend in several Rust projects.
- **Strengths:** Pure Rust (no unsafe FFI to C), ACID transactions, very simple API, competitive performance with LMDB.
- **Source:** https://github.com/cberner/redb

### Fjall / Lsm-tree
- **What it is:** LSM-tree based key-value storage engine written in pure Rust. `fjall` is the database layer built on top of the `lsm-tree` crate.
- **Production status:** Active development (2023–2025). Not yet production-hardened at scale.
- **Source:** https://github.com/fjall-rs/fjall

### Skytable
- **What it is:** In-memory + persistent NoSQL database written in Rust, targeting Redis-like use cases with more data modeling.
- **Production status:** v0.8+ available; not yet widely adopted.
- **Source:** https://github.com/skytable/skytable

---

## Rust in Existing Database Systems

### Apache Arrow & Arrow Flight (arrow-rs)
- The official Rust implementation of Apache Arrow (`arrow-rs`) is one of the most important Rust projects in the data ecosystem.
- Used as the in-memory columnar format by DataFusion, Ballista, Databend, GreptimeDB, Delta Lake Rust, and many others.
- **Flight SQL** (the Arrow-native database protocol over gRPC) has a mature Rust server/client implementation.
- **Source:** https://github.com/apache/arrow-rs

### Apache DataFusion
- **What it is:** Embeddable, extensible SQL query engine written in Rust, using Apache Arrow as its in-memory format.
- **Production status:** Apache top-level project. Used in production by InfluxDB IOx, Databend, HoraeDB, GlueSQL, Delta-rs, and many others.
- **Key features:** Vectorized execution, parallel query processing, pluggable catalog/storage, physical and logical plan APIs, built-in optimizer.
- **This is arguably the most impactful Rust project in databases today** — it serves as the query layer for an entire ecosystem of new databases.
- **Source:** https://github.com/apache/datafusion

### InfluxDB IOx
- InfluxDB v3 (IOx) rewrote the entire storage and query engine in Rust, abandoning the Go-based InfluxDB v2 codebase.
- Uses DataFusion for query execution, Apache Arrow for in-memory data, and Parquet for on-disk storage on object stores.
- **Significance:** Demonstrates that even a mature, established database (InfluxDB) found Rust compelling enough to fully rewrite their engine.
- **Source:** https://github.com/influxdata/influxdb_iox

### Delta Lake Rust (`delta-rs`)
- Rust-native implementation of the Delta Lake protocol.
- Enables reading/writing Delta tables without Spark — from Python (via PyO3 bindings), Rust, or any Arrow-compatible tool.
- Used in production by many teams running lakehouse architectures.
- **Source:** https://github.com/delta-io/delta-rs

### Apache Iceberg Rust (`iceberg-rust`)
- Official Rust implementation of the Apache Iceberg table format specification.
- Enables reading/writing Iceberg tables from Rust and Python without JVM dependency.
- Still maturing (2023–2025) but strategically important.
- **Source:** https://github.com/apache/iceberg-rust

### Tantivy (Full-Text Search)
- **What it is:** Full-text search engine library written in Rust, inspired by Apache Lucene but orders of magnitude faster for indexing.
- Used as the search backend for **Quickwit** and other search tools.
- **Source:** https://github.com/quickwit-oss/tantivy

### Quickwit
- Cloud-native distributed search engine built on Tantivy (Rust). Positioned as an Elasticsearch alternative optimized for log data on object storage.
- **Source:** https://github.com/quickwit-oss/quickwit

### Parquet2 / Parquet-rs
- `parquet` crate in `arrow-rs` is a high-performance Parquet reader/writer.
- Critical infrastructure: every Rust-based analytics system uses it.
- **Source:** https://github.com/apache/arrow-rs/tree/master/parquet

### PostgreSQL Extensions in Rust (pgrx)
- `pgrx` is a framework for writing PostgreSQL extensions in Rust instead of C.
- Enables safe, high-performance custom data types, index methods, and functions within Postgres.
- Projects using pgrx: **pgvector-rs**, **pg_analytics** (from ParadeDB), **pg_lakehouse**.
- **ParadeDB** is building a full Postgres-native analytical and search layer using pgrx + Tantivy.
- **Source:** https://github.com/pgcentralfoundation/pgrx | https://github.com/paradedb/paradedb

### RocksDB Rust Bindings (`rust-rocksdb`)
- FFI bindings to RocksDB from Rust. Widely used but these are bindings to C++, not a Rust reimplementation.
- Known pain point: compilation times, cross-compilation difficulty, and unsafe FFI boundaries.
- This is a significant gap — a native Rust LSM-tree engine at RocksDB's level does not yet exist.

---

## Data Processing / ETL in Rust

### Apache Arrow DataFusion (again)
- Beyond being a query engine, DataFusion is used as an ETL execution engine for transforming data between formats (CSV → Parquet, JSON → Arrow, etc.).

### Arroyo
- **What it is:** Distributed stream processing engine written in Rust, designed as an Apache Flink alternative with a SQL interface.
- **Production status:** v0.10+ in 2025. Open-source with a commercial offering.
- **Key features:** Rust-native pipelines, Kafka/Kinesis sources/sinks, windowed aggregations, SQL interface, WebAssembly for user-defined functions.
- **Gap it fills:** Flink is operationally heavy (JVM, complex state backend). Arroyo aims to be lightweight and embeddable.
- **Source:** https://github.com/ArroyoSystems/arroyo

### Fluvio
- **What it is:** Distributed streaming platform written in Rust — positions itself as a Kafka alternative with a much simpler operational model.
- **Production status:** v0.x, active development. Not yet at Kafka's ecosystem breadth.
- **Strengths:** Single binary deployment, WebAssembly-based stream processing (SmartModules), low latency, built-in CLI.
- **Source:** https://github.com/infinyon/fluvio

### Redpanda
- Written in C++ (not Rust), but worth noting as the context for what "Kafka but fast and simple" looks like when rewritten. It demonstrates the market demand.
- Redpanda's success (reaching $100M+ ARR) validates that there is massive appetite for Kafka replacements.

### Ballista (DataFusion Distributed)
- Distributed query execution engine built on DataFusion. Allows DataFusion queries to run across a cluster.
- **Status:** Part of the DataFusion project. Less mature than the single-node DataFusion; not widely production-deployed.
- **Source:** https://github.com/apache/datafusion-ballista

### Polars
- **What it is:** DataFrame library and in-process query engine written in Rust with Python and R bindings.
- **Production status:** Extremely widely used. v1.0 in 2024. One of the fastest-growing data tools in the Python ecosystem.
- **Key features:** Lazy evaluation, query optimizer, vectorized execution, Arrow-native, multi-threaded.
- **Why it matters:** Polars is arguably the most successful "Rust data tool used by non-Rust developers" — it displaced Pandas in many workloads purely on performance.
- **Source:** https://github.com/pola-rs/polars | https://pola.rs

### DataFusion Comet (Apache Spark accelerator)
- Apache DataFusion Comet is a Spark plugin that replaces Spark's native execution engine with DataFusion's vectorized Rust engine.
- Significant performance gains (2x–5x reported) for Spark workloads with zero code changes.
- **Source:** https://github.com/apache/datafusion-comet

### Pipelinewise / custom ETL
- No dominant "Airbyte in Rust" exists yet — this is a notable gap (see Gaps section).

---

## Time-Series Databases in Rust

### What Exists

| Project | Status | Notes |
|---|---|---|
| InfluxDB IOx (v3) | Production | Full rewrite in Rust; most mature Rust TSDB |
| GreptimeDB | v0.9+ 2025 | Cloud-native, SQL+PromQL, growing |
| HoraeDB / CeresDB | Production (Ant) | Good internally; limited external adoption |
| Cnosdb | Active | Pure Rust TSDB, IoT focused |
| Tonbo | Early | LSM-based embedded TSDB in Rust |

### What's Missing
- **No Rust-native equivalent of TimescaleDB** (Postgres extension for time-series at scale). TimescaleDB is C. A `pgrx`-based TimescaleDB-like extension in Rust would be transformative.
- **No dominant Rust TSDB for edge/IoT** comparable to what TDengine is in C. Embedded time-series with sub-millisecond writes and a tiny binary footprint is underserved.
- **Prometheus-compatible TSDB in Rust:** VictoriaMetrics (Go) dominates here. No serious Rust competitor exists.

---

## Vector Databases in Rust

### Qdrant (Production Leader)
- The most production-ready vector database written in any language that is not C++/Go.
- Full feature set: HNSW indexing, scalar quantization, product quantization, binary quantization, sparse vectors, payload filtering, distributed sharding.
- Python, JavaScript, Go, Java, and Rust client libraries.
- **Source:** https://github.com/qdrant/qdrant

### LanceDB
- Serverless, embedded-first. Uses the Lance columnar format (also Rust) which stores vectors alongside other data columns efficiently.
- **Source:** https://github.com/lancedb/lancedb

### Milvus (not Rust)
- The dominant vector database (Go + C++) for scale. Important as context — Rust has no equivalent at Milvus's scale/feature-level yet.

### Gaps in Rust Vector Databases
- **No Rust vector database with Postgres-compatible SQL + vector search** (pgvector is C; a pgrx-based replacement would be compelling).
- **No Rust-native graph + vector hybrid** — knowledge graph traversal combined with semantic vector search in a single Rust engine is an open space.

---

## Columnar Storage / OLAP Engines in Rust

### Apache DataFusion
- The backbone of Rust-based OLAP. Provides logical/physical planning, vectorized execution, and pluggable catalogs.
- Used by: Databend, InfluxDB IOx, HoraeDB, GlueSQL, Delta-rs query layer, DataFusion Comet.
- **Source:** https://github.com/apache/datafusion

### Databend
- Full OLAP warehouse built on DataFusion + Arrow. Object storage native.
- **Source:** https://github.com/datafuselabs/databend

### DuckDB (not Rust, but critical context)
- Written in C++. The most popular embedded OLAP database. Often cited as "what a Rust version of this could look like."
- DuckDB's success has directly inspired interest in a Rust alternative or Rust extensions. No credible Rust replacement exists yet.

### Velox (not Rust)
- Meta's C++ vectorized execution engine. Used in Presto, Spark. Demonstrates the market for high-performance native execution engines — a space where Rust could compete.

### Glaredb
- Distributed SQL query engine written in Rust that federates queries across data sources (Postgres, MySQL, BigQuery, S3 Parquet, etc.).
- **Source:** https://github.com/GlareDB/glaredb

### SpiceAI
- Data acceleration layer written in Rust; uses DataFusion + Arrow Flight to create an in-process cache of remote data.
- **Source:** https://github.com/spiceai/spiceai

---

## SQLite — Rust Reimplementation or Extension

### Limbo
- **What it is:** SQLite reimplementation in Rust, initiated by Pekka Enberg (creator of the original Chiselstore). Developed under Turso.
- **Status as of mid-2025:** Under active development. Core read path working; write path and full SQL coverage in progress.
- **Significance:** The most serious attempt to build a fully compatible, pure-Rust SQLite replacement. If successful, enables safe embedding, WASM targeting, async I/O, and better extension APIs.
- **Source:** https://github.com/tursodatabase/limbo

### SQLite via pgrx
- Not applicable (pgrx is for Postgres extensions).

### `rusqlite`
- Rust FFI bindings to SQLite (not a reimplementation). Widely used but carries the same cross-compilation pain as any C FFI.
- **Source:** https://github.com/rusqlite/rusqlite

### `sqlite3-parser` in Rust
- There are Rust crates for parsing SQLite's SQL dialect, used by Limbo and other tools.

### Why a Rust SQLite Matters
- SQLite runs on billions of devices. A memory-safe replacement or a compatible extension layer would be a genuine industry contribution.
- WASM compilation: Rust compiles cleanly to WASM; SQLite's C does too, but with more friction. A Rust SQLite could be the foundation for browser-native databases.
- Async I/O: SQLite is synchronous by design. A Rust reimplementation could natively support async I/O via Tokio without the awkward wrappers.

---

## Gaps — High Value Missing Projects

### 1. Native Rust LSM-Tree Storage Engine (RocksDB-class)
- **The gap:** RocksDB is C++. Every Rust project that needs an LSM-tree engine either wraps RocksDB via FFI (compilation pain, unsafe, poor cross-compilation) or uses immature alternatives (`fjall`, `lsm-tree` crates).
- **Evidence:** TiKV uses `rust-rocksdb` (C++ FFI). SurrealDB uses RocksDB via FFI. Multiple projects have noted compilation time and cross-compilation as blockers.
- **What's needed:** A pure-Rust, production-grade LSM-tree engine with compaction strategies (Leveled, STCS, TWCS), bloom filters, block cache, and write-ahead log.
- **Value:** Would become the storage backend for the entire Rust database ecosystem.
- **Closest existing:** `fjall`/`lsm-tree` (early stage), `redb` (B-tree, not LSM).

### 2. Rust-Native Kafka-Compatible Message Broker
- **The gap:** Fluvio exists but lacks Kafka protocol compatibility. Redpanda (C++) demonstrates massive demand for a non-JVM Kafka replacement.
- **What's needed:** A broker that speaks the Kafka wire protocol natively (so existing Kafka clients work without changes) but is written in Rust for better memory safety and resource efficiency.
- **Evidence:** Redpanda's funding and growth ($100M+ ARR) proves market demand. The Kafka ecosystem is enormous — a Rust-native compatible broker with Kafka's protocol would be immediately adoptable.
- **Value:** Deploy with 1/10th the RAM of Kafka; no JVM GC pauses; memory safe.

### 3. DuckDB-Equivalent in Rust (Embedded OLAP)
- **The gap:** DuckDB is C++ and dominant. No Rust embedded OLAP with comparable feature set and ecosystem exists.
- **Evidence:** DuckDB is the most-discussed database in the data engineering community (2023–2025). Its API (SQL over files, direct Parquet/JSON/CSV query) is widely loved.
- **What's needed:** An embeddable OLAP engine in Rust with: columnar execution, direct S3/file querying, Python bindings, and compatibility with Arrow/Parquet.
- **Note:** DataFusion is the building block, but it requires significant work to expose as a DuckDB-like product. GlareDB is the closest attempt.

### 4. Prometheus-Compatible Time-Series Database in Rust
- **The gap:** VictoriaMetrics (Go) and Prometheus itself (Go) dominate. No serious Rust TSDB with native PromQL and remote_write/read compatibility.
- **Evidence:** VictoriaMetrics is one of the fastest-growing observability tools, demonstrating demand for a high-performance Prometheus-compatible TSDB. The Rust ecosystem has GreptimeDB (supports PromQL) but lacks a drop-in Prometheus replacement.
- **Value:** Metrics/observability is a massive market. A Rust TSDB that acts as a drop-in Prometheus replacement with better performance would have instant adoption.

### 5. ETL / ELT Framework in Rust (Airbyte-class)
- **The gap:** No Rust-native ETL connector framework exists. Airbyte, Fivetran, and dbt are JVM/Python.
- **Evidence:** Data engineers constantly complain about Python's performance in transformation steps and the complexity of the JVM for simple connector logic.
- **What's needed:** A connector specification + runtime in Rust, where connectors are compiled Rust (or WASM) plugins, with a lightweight orchestrator.
- **Value:** 10x lower resource usage per connector; WASM-based connectors could run in sandboxed environments.

### 6. Async Postgres Wire Protocol Server Library
- **The gap:** Building a custom Postgres-wire-protocol-compatible server (like PgBouncer, a proxy, or a custom database that speaks Postgres) requires either wrapping libpq (C) or using incomplete Rust libraries.
- **Evidence:** Multiple projects (RisingWave, Databend, Neon, DataFusion query servers) have had to independently implement the Postgres wire protocol in Rust. There is no single high-quality, maintained crate for this.
- **What's needed:** A production-quality `tokio`-based async Postgres server protocol library.
- **Closest existing:** `pgwire` crate (functional but limited), `rust-postgres` client (not a server library).

### 7. Pure Rust B-Tree / B+-Tree Storage Engine (SQLite-class)
- **The gap:** `redb` is good but API-limited. A full B-Tree engine with variable-length pages, row-level locking, and MVCC suitable for use as a general embedded relational storage does not exist in pure Rust.

### 8. Change Data Capture (CDC) Engine in Rust
- **The gap:** Debezium (Java) dominates CDC. No Rust CDC engine with connectors for Postgres/MySQL logical replication exists.
- **Value:** CDC is critical infrastructure for data pipelines. A memory-safe, low-overhead CDC engine that reads Postgres WAL or MySQL binlog and streams to Kafka/Arrow Flight would be widely adopted.

---

## Community Discussions

### Reddit and Hacker News Themes (through mid-2025)

**"Rust database that should exist"**
- HN threads on DataFusion repeatedly surface the question: "When does DataFusion become DuckDB?"
  The answer from maintainers: DataFusion is a library, not a product. Someone needs to build the product layer.
- Multiple Reddit r/rust threads ask about LSM-tree alternatives to RocksDB in pure Rust.
  The consensus: `fjall` is promising but not production-ready; most teams still wrap RocksDB.

**"What's slow in our data stack"**
- r/dataengineering threads frequently cite: Airbyte connector startup time, Python transformation overhead, JVM heap tuning for Kafka/Flink.
- Common complaint: "Kafka requires 3 ZooKeeper nodes and 3 Kafka brokers for HA — this is insane for small teams."
- Redpanda is the most-cited answer; a Rust Kafka-compatible equivalent would be equally well-received.

**"InfluxDB IOx rewrite discussion"**
- The IOx rewrite (from Go to Rust) generated significant HN discussion. Key takeaways from the team:
  - Rust's ownership model caught races and bugs at compile time that had been production incidents in Go.
  - Arrow + DataFusion gave them a 10x+ query performance improvement over their Go-based engine.
  - Compile times were the main complaint.
- Reference: Paul Dix (InfluxDB CEO) blog posts, 2022–2023.

**"Polars vs Pandas"**
- r/Python and r/dataengineering have hundreds of threads comparing Polars (Rust) to Pandas.
  Polars consistently wins on performance; the main adoption blocker is API familiarity and ecosystem integrations (not performance).
- This is perhaps the strongest real-world proof that Rust data tools get adopted when they solve a genuine pain point.

**"Why isn't there a Rust Elasticsearch"**
- Quickwit threads on HN show interest but also a recurring question about whether Elasticsearch's feature set (not just performance) justifies a Rust rewrite.
- Tantivy is considered excellent but "Tantivy is to Lucene as LLVM is to GCC — a great library, but you still need someone to build the database on top."

**"pgvector is slow"**
- Growing number of complaints (2024–2025) that pgvector's HNSW implementation is slower than dedicated vector databases.
- Several discussions about building a pgrx-based replacement that uses Rust's SIMD intrinsics for distance computation.
- ParadeDB's `pg_search` extension (Rust via pgrx) is seen as proof this approach works.

---

## Failed Attempts

### 1. Actix-based Databases (various)
- Several hobby projects attempted to build databases on Actix (actor framework) around 2019–2021.
- Most failed because: actor overhead for database hot paths, Actix's API churn (the maintainer briefly abandoned it), and lack of persistence layers.
- **Lesson:** Database internals require fine-grained control that actor abstractions obscure. Most serious Rust databases use Tokio directly.

### 2. Early IndraDB
- IndraDB was an early Rust graph database (2016–2018) with some traction.
- Faded due to: limited query language, no distributed mode, and competition from more feature-complete options.
- The codebase is still on GitHub but largely unmaintained.
- **Source:** https://github.com/indradb/indradb

### 3. Sled
- `sled` is an embedded key-value database in pure Rust that was very promising (2018–2021, ~7k GitHub stars).
- **What went wrong:** The author (Tyler Neely) has been open about the project's difficulties:
  - Correctness issues discovered through extensive fuzzing (jepsen-style testing).
  - The complexity of building a concurrent, crash-safe storage engine from scratch in Rust — even Rust's type system doesn't prevent all subtle concurrency bugs at the storage level.
  - "Sled is beta-quality. I wouldn't use it for anything important." (author's own words as of 2023)
  - Active development appears to have stalled.
- **Lesson:** Storage engine correctness is brutally hard. Even talented Rust engineers with years of effort can produce a project not ready for production. This validates the approach of building on proven storage (RocksDB, LMDB) rather than rolling new engines from scratch.
- **Source:** https://github.com/spacejam/sled | https://sled.rs

### 4. Skytable Stagnation
- Skytable had ambitions of being a high-performance Redis alternative in Rust. The project has had slow development velocity since ~2022 and has not gained significant traction.

### 5. Ballista (Distributed DataFusion)
- While not "failed," Ballista has struggled to find production users. The challenge: DataFusion single-node is already very fast; the distributed version requires solving hard distributed systems problems (fault tolerance, scheduling, shuffle) that the core team hasn't prioritized.
- Most production users who need distributed DataFusion have built their own orchestration rather than adopting Ballista.

### 6. TensorBase
- Early Rust ClickHouse-like OLAP database. Development stalled around 2022. No clear reason given; likely a combination of limited maintainer resources and the difficulty of competing with ClickHouse's maturity.
- **Source:** https://github.com/tensorbase/tensorbase

---

## Top Candidates for New Rust Database Projects

Ranked by: (a) clear market demand, (b) Rust-specific advantages, (c) gap from existing Rust solutions.

### Rank 1: Native Rust LSM-Tree Storage Engine
- **Why #1:** Would unblock the entire Rust database ecosystem. Every Rust database that needs a write-optimized key-value store currently uses RocksDB via FFI.
- **Rust advantage:** Memory safety in the hot write path; no GC pauses; async I/O native support; better cross-compilation.
- **Monetization:** Dual-license like RocksDB or as managed storage service.
- **Risk:** Sled tried and stalled. Requires deep storage engineering expertise. Correctness testing (fuzzing, Jepsen) is non-negotiable.

### Rank 2: Kafka-Protocol-Compatible Broker in Rust
- **Why #2:** Massive existing Kafka ecosystem; Redpanda proved the market is real and enormous.
- **Rust advantage:** No JVM GC pauses; memory safety; faster cold start; WASM extension model possible.
- **Differentiator vs. Redpanda:** Rust's safety story + WASM-based stream processing plugins (like Fluvio's SmartModules but with Kafka compatibility).
- **Risk:** Kafka protocol compatibility is complex (50+ API versions). Significant engineering investment before MVP.

### Rank 3: Embedded OLAP / DuckDB Alternative
- **Why #3:** DuckDB is the hottest database in data engineering. A Rust-native equivalent with Python bindings would immediately attract the data science community.
- **Rust advantage:** WASM compilation (DuckDB WASM exists but is C++); safety in complex query execution; easier embedding in Rust applications.
- **Building blocks exist:** DataFusion + Arrow + Parquet. This is a "build on top of" project, not ground-up.
- **Risk:** DuckDB is excellent and free. Must offer genuine differentiation (e.g., better WASM, Rust-native embedding, custom extensions via pgrx-style framework).

### Rank 4: Change Data Capture (CDC) Engine
- **Why #4:** Debezium is Java, resource-heavy. CDC is used in nearly every modern data stack.
- **Rust advantage:** Low-overhead continuous process; memory safety in protocol parsing (Postgres logical replication, MySQL binlog parsing); small binary for edge deployments.
- **Differentiation:** Could output directly to Arrow Flight / Delta Lake / Iceberg rather than just Kafka.
- **Risk:** Protocol parsing for Postgres WAL and MySQL binlog is complex and version-sensitive.

### Rank 5: Prometheus-Compatible TSDB
- **Why #5:** VictoriaMetrics is fast but Go-based. Prometheus itself is limited at scale. A Rust drop-in with the remote_write/read API, PromQL, and better compaction would attract the observability market.
- **Rust advantage:** Predictable latency (no GC), compact memory layout for time-series data, async I/O.
- **Building blocks:** GreptimeDB has done some of this work; extracting/building a standalone component is feasible.

### Rank 6: Async Postgres Wire Protocol Server Library
- **Why #6:** Enabling factor for every project that wants to be "Postgres compatible." Low-effort contribution with high ecosystem leverage.
- **Rust advantage:** Tokio-native async; can be the foundation for query routers, proxies, and custom databases.
- **Risk:** Low — this is a well-scoped library problem, not a full database problem.

### Rank 7: SQLite Full Reimplementation (Limbo completion)
- **Why #7:** Limbo is already underway. Contributing to / accelerating Limbo could be more valuable than starting fresh.
- **Rust advantage:** WASM-first, async I/O, memory safety for extension API.
- **Risk:** SQLite compatibility is enormous — the test suite alone is millions of lines. Full compatibility will take years.

---

## Sources

### Rust Database Projects — GitHub Repositories
- SurrealDB: https://github.com/surrealdb/surrealdb
- TiKV: https://github.com/tikv/tikv
- Neon: https://github.com/neondatabase/neon
- HoraeDB (CeresDB): https://github.com/apache/horaedb
- Databend: https://github.com/datafuselabs/databend
- Qdrant: https://github.com/qdrant/qdrant
- LanceDB: https://github.com/lancedb/lancedb
- GreptimeDB: https://github.com/GreptimeTeam/greptimedb
- RisingWave: https://github.com/risingwavelabs/risingwave
- Redb: https://github.com/cberner/redb
- Fjall: https://github.com/fjall-rs/fjall
- Skytable: https://github.com/skytable/skytable
- GlueSQL: https://github.com/gluesql/gluesql
- Limbo (Rust SQLite): https://github.com/tursodatabase/limbo
- Sled (stalled): https://github.com/spacejam/sled
- IndraDB: https://github.com/indradb/indradb
- TensorBase: https://github.com/tensorbase/tensorbase

### Ecosystem Infrastructure
- Apache DataFusion: https://github.com/apache/datafusion
- Apache Arrow Rust: https://github.com/apache/arrow-rs
- Apache Iceberg Rust: https://github.com/apache/iceberg-rust
- Delta Lake Rust: https://github.com/delta-io/delta-rs
- InfluxDB IOx: https://github.com/influxdata/influxdb_iox
- Tantivy: https://github.com/quickwit-oss/tantivy
- Quickwit: https://github.com/quickwit-oss/quickwit
- DataFusion Comet: https://github.com/apache/datafusion-comet
- DataFusion Ballista: https://github.com/apache/datafusion-ballista
- pgrx (Postgres extensions in Rust): https://github.com/pgcentralfoundation/pgrx
- ParadeDB (pgrx-based): https://github.com/paradedb/paradedb
- Polars: https://github.com/pola-rs/polars

### Stream Processing / ETL
- Arroyo: https://github.com/ArroyoSystems/arroyo
- Fluvio: https://github.com/infinyon/fluvio
- SpiceAI: https://github.com/spiceai/spiceai
- GlareDB: https://github.com/GlareDB/glaredb

### Supporting Documentation & Articles
- Neon architecture blog: https://neon.tech/blog/architecture
- SurrealDB docs: https://surrealdb.com/docs
- TiKV website: https://tikv.org
- Polars website: https://pola.rs
- InfluxDB IOx design docs: https://github.com/influxdata/influxdb_iox/tree/main/docs
- sled author blog (Tyler Neely): https://sled.rs
- pgrx documentation: https://github.com/pgcentralfoundation/pgrx/tree/develop/docs

### Community Discussion Forums (for independent verification)
- Hacker News search for "DataFusion": https://hn.algolia.com/?q=datafusion
- Hacker News search for "InfluxDB IOx Rust": https://hn.algolia.com/?q=influxdb+iox+rust
- Hacker News search for "Polars Rust": https://hn.algolia.com/?q=polars+rust
- Reddit r/rust database discussions: https://www.reddit.com/r/rust/search/?q=database
- Reddit r/dataengineering Kafka complaints: https://www.reddit.com/r/dataengineering/search/?q=kafka+alternative

---

*Research compiled from knowledge base through August 2025. All GitHub URLs and project statuses reflect the state as of that date. Independent verification recommended for current project activity and production readiness.*
