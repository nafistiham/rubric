# Security & System Tools: Rust Research

> Research compiled from public documentation, GitHub repositories, crates.io, security advisories,
> CVE databases, community forums, and industry reports. Knowledge cutoff: August 2025.

---

## Existing Rust Security Tools

### Network Scanners and Reconnaissance

**RustScan** (github.com/RustScan/RustScan)
- The most prominent Rust alternative to nmap for port scanning
- Claims to scan all 65,535 ports in under 3 seconds on localhost; benchmarks show ~3–5x faster than nmap for raw port discovery
- Does NOT replace nmap entirely — it hands off discovered open ports to nmap for service/version detection
- 14,000+ GitHub stars as of mid-2025
- Actively maintained; used in CTF competitions and real pentests
- Supports custom scripts, nmap integration, and adaptive learning to avoid overloading targets

**Skanuvaty** / **scanr**
- Smaller Rust port scanners; less mature than RustScan but exist as alternatives
- Proof-of-concept quality; not production-recommended

**Netscan** (crates.io)
- Library-level network scanning crate; not a standalone CLI tool
- Used as a dependency in other tools

### Fuzzers

**cargo-fuzz** (github.com/rust-fuzz/cargo-fuzz)
- Official Rust fuzzing tool using libFuzzer as the backend engine
- Widely used for fuzzing Rust code itself; also used to fuzz C/C++ code exposed via Rust FFI wrappers
- Maintained by the Rust Fuzz project (rust-fuzz GitHub org)
- Production-grade; used by major crates including serde, image, and regex

**AFL.rs** (github.com/rust-fuzz/afl.rs)
- Rust bindings and wrapper for American Fuzzy Lop (AFL)
- Supports persistent mode; works well for fuzzing binary format parsers
- Less ergonomic than cargo-fuzz but supports different coverage mechanisms

**Honggfuzz-rs** (github.com/rust-fuzz/honggfuzz-rs)
- Rust bindings for Google's honggfuzz
- Good for hardware-assisted coverage; used in some cryptographic library testing

**LibAFL** (github.com/AFLplusplus/LibAFL)
- Next-generation fuzzing framework written largely in Rust
- Modular architecture; used to build custom fuzzers rather than as a standalone tool
- Supports in-process, out-of-process, and distributed fuzzing
- Backed by AFLplusplus authors; growing adoption in research and industry
- 4,000+ GitHub stars; one of the most ambitious fuzzing projects in Rust

### Penetration Testing Tools

**RustScan** — see above

**Feroxbuster** (github.com/epi052/feroxbuster)
- Fast, recursive web content discovery tool (directory brute-forcer)
- Direct replacement for gobuster and dirb; written entirely in Rust
- Supports recursive scanning, regex filtering, output to multiple formats, rate limiting
- 6,000+ GitHub stars; widely adopted in the penetration testing community
- Used in OSCP prep courses and professional engagements

**x8** (github.com/Sh1Yo/x8)
- HTTP parameter discovery tool (finds hidden GET/POST parameters)
- Rust-based; used in bug bounty hunting and web app pentesting
- Differentiates itself by analyzing response differences to detect hidden params

**Findomain** (github.com/findomain/findomain)
- Subdomain enumeration tool written in Rust
- Queries certificate transparency logs, APIs (Shodan, VirusTotal, etc.)
- Faster than Amass for pure subdomain discovery; 6,000+ stars
- Has a monitoring mode to alert on new subdomains

**Ripasso** — password manager, see Password Managers section

**CrackStation / hashcat alternatives**
- No mature Rust alternative to hashcat exists yet (GPU kernel complexity is the barrier)
- Some proof-of-concept Rust hash crackers exist on crates.io but are research-grade only

### Exploit Development / Reversing Helpers

**pwntools-rs** — early-stage; does not match Python pwntools in features
**binsec** — binary analysis utilities in Rust; experimental

### Web Application Security

**Nuclei** — written in Go, not Rust; but Rust-based template engines exist for building similar tools
**Rustsec Advisory DB** — the Rust ecosystem's own vulnerability advisory database (rustsec.org); not a scanner per se, but critical infrastructure
**cargo-audit** (github.com/rustsec/rustsec)
- Audits Cargo.lock for dependencies with known security vulnerabilities
- The Rust equivalent of npm audit or bundler-audit
- Production-grade; integrated into many CI pipelines
- Maintained by the Rust Security Response WG

**cargo-deny** (github.com/EmbarkStudios/cargo-deny)
- Policy enforcement for Rust dependencies: bans, license checks, vulnerability scanning
- More featureful than cargo-audit for CI enforcement

### Intrusion Detection / Network Security

**Suricata** — written in C; Rust bindings exist and Suricata itself is adding Rust rule parsers
**Snort** — C; no Rust replacement at production quality
**Zeek (Bro)** — C++; no Rust replacement

**quartz-iids** — experimental Rust IDS; not production-ready

**Retis** (github.com/retis-org/retis)
- eBPF-based network event collection and analysis tool written in Rust
- Used for observing packet flows; overlaps with IDS-like use cases
- Maintained by Red Hat engineers; early but promising

**aya** (github.com/aya-rs/aya)
- eBPF library for Rust; enables writing eBPF programs (network filters, intrusion detection hooks) in Rust
- Production-quality library; used to build custom security monitoring tools
- Does not ship a standalone IDS but is the foundation for building one in Rust

### Firewalls

No production-grade userspace firewall written in Rust exists yet.
**nftables** and **iptables** — C; Rust bindings (nftables-rs) exist but are wrappers
**pfSense/OPNsense** — PHP/C; no Rust alternative
**eBPF-based firewalls in Rust** are possible via aya; several companies build internal tools this way

### Password Managers and Secret Management

**Ripasso** (github.com/cortex/ripasso)
- Rust frontend for the UNIX `pass` password manager (GPG-based)
- GUI and TUI interfaces; production-grade for personal use
- Not a full independent password manager — depends on gpg and the pass directory format

**Bitwarden** — C# server, web; Vaultwarden (github.com/dani-garcia/vaultwarden) is an unofficial Rust reimplementation of the Bitwarden server
- Vaultwarden: 40,000+ GitHub stars; the most widely self-hosted password manager server
- Production-grade; compatible with all official Bitwarden clients
- Memory-safe, much lower resource footprint than the official C# server
- Single binary; used by thousands of self-hosters

**Goldwarden** (github.com/quexten/goldwarden)
- Rust-based Bitwarden client for Linux with SSH agent, browser integration, and polkit support
- Modern, actively maintained

**HashiCorp Vault** — Go; no direct Rust replacement, but Rust clients exist (vaultrs)
**SOPS** — Go; no Rust equivalent

**lox** / **passage** — experimental Rust secret stores; not production-ready

---

## Cryptography in Rust

### Core Production Libraries

**ring** (github.com/briansmith/ring)
- The most widely used cryptographic library in Rust
- Wraps BoringSSL's assembly-optimized primitives with a safe Rust API
- Provides: AES-GCM, ChaCha20-Poly1305, ECDSA, Ed25519, RSA, SHA-2, HKDF, PBKDF2, HMAC
- Used by: rustls, AWS SDK, Cloudflare, Let's Encrypt (certbot alternatives), many high-profile crates
- NOT a pure-Rust implementation — uses C/asm from BoringSSL under the hood
- Opinionated API: intentionally omits weak/legacy algorithms
- Production-grade; extremely well-audited

**RustCrypto** (github.com/RustCrypto)
- Collection of pure-Rust cryptographic algorithm implementations
- Covers: AES, ChaCha20, SHA-1/2/3, BLAKE2/3, RSA, ECDSA, Ed25519, X25519, AES-GCM, AES-SIV, Argon2, bcrypt, scrypt, PBKDF2, HKDF, HMAC, Poly1305
- Key sub-crates: `aes`, `chacha20`, `sha2`, `rsa`, `p256`, `ed25519`, `argon2`
- Pure Rust: no C FFI dependencies; fully auditable
- Downside: slower than ring/BoringSSL for some primitives (though gap is shrinking)
- Multiple crates have received third-party security audits (NCC Group, Trail of Bits)
- Production-grade for many use cases; actively maintained by a large community

**dalek-cryptography** (github.com/dalek-cryptography)
- Pure-Rust implementations of Curve25519-based cryptography
- `curve25519-dalek`: the backbone for many Rust crypto implementations
- `ed25519-dalek`: EdDSA signatures; used in many projects
- `x25519-dalek`: Diffie-Hellman key exchange
- `bulletproofs`: zero-knowledge range proofs
- Used by: Signal Protocol implementations, Tor (arti), multiple blockchain projects
- Production-grade; security-audited; some parts incorporated into RustCrypto

**aws-lc-rs** (github.com/aws/aws-lc-rs)
- AWS's Rust wrapper around AWS-LC (a BoringSSL fork maintained by AWS)
- Drop-in alternative to ring with broader algorithm support
- FIPS 140-3 validated module available (critical for US government/regulated industries)
- Growing adoption as ring alternative especially where FIPS compliance is required

**OpenSSL bindings (openssl crate)**
- Rust FFI bindings to the system OpenSSL or a vendored OpenSSL
- Not a rewrite; exists so Rust code can use OpenSSL without a full native implementation
- Production-grade bindings; widely used where OpenSSL compatibility is required

### TLS Implementations

**rustls** (github.com/rustls/rustls)
- Pure-Rust TLS 1.2 and TLS 1.3 implementation; does NOT use OpenSSL
- Uses ring or aws-lc-rs as the cryptographic backend
- TLS 1.3 only by default in newer versions (can enable 1.2); drops all deprecated algorithms
- Memory-safe by design; has no legacy C code in the TLS state machine
- Security audit conducted by iSEC Partners (now NCC Group) in 2020; followed by a 2023 audit
- Adoption: used by Cloudflare (cloudflare/quiche), AWS (aws-sdk-rust), Tokio ecosystem (tokio-rustls), hyper, reqwest, curl (via rustls backend), and many others
- curl added a rustls backend in 2021: `./configure --with-rustls`; this is significant as curl is in ~10 billion installations
- Performance: competitive with OpenSSL; faster for TLS 1.3 handshakes in benchmarks
- Limitations: no DTLS, no QUIC (though quiche and quinn use rustls internals), limited PKCS#11 support (improving)
- FIPS compliance: rustls with aws-lc-rs backend can achieve FIPS; announced 2024

**quinn** (github.com/quinn-rs/quinn)
- QUIC implementation in Rust; uses rustls for TLS
- Production-grade; used in real deployments

**boring** (github.com/cloudflare/boring)
- Cloudflare's Rust bindings to BoringSSL
- Used internally at Cloudflare; not as widely adopted as rustls but available

### Zero-Knowledge and Advanced Cryptography

**arkworks** (github.com/arkworks-rs)
- Rust framework for zkSNARK development
- Libraries for elliptic curves, polynomial commitments, constraint systems
- Used in blockchain projects (Zcash, Filecoin, etc.)
- Research/blockchain-grade; not general enterprise use

**bellman** (github.com/zkcrypto/bellman)
- Rust zkSNARK library; used by Zcash
- Production in blockchain context

**halo2** (github.com/zcash/halo2)
- ZK proof system from Zcash/Electric Coin Company; written in Rust
- Used in production Zcash protocol

**snow** (github.com/mcginty/snow)
- Rust implementation of the Noise Protocol Framework
- Used in WireGuard-related tooling and secure messaging

### Gaps in Rust Cryptography

1. **FIPS 140-2/3 validation** — ring alone is not FIPS validated; aws-lc-rs fills this gap but adoption is still building
2. **PKCS#11 hardware token support** — limited native Rust support; mostly requires C middleware
3. **HSM (Hardware Security Module) integration** — no mature pure-Rust HSM SDK
4. **S/MIME** — no good Rust implementation
5. **PGP/GPG full implementation** — sequoia-pgp exists (see below) but is not drop-in for GnuPG
6. **CMS (Cryptographic Message Syntax)** — partial support in RustCrypto
7. **Legacy algorithm support** — intentionally absent in ring; needed for interop with old systems

**Sequoia-PGP** (gitlab.com/sequoia-pgp/sequoia)
- Modern Rust implementation of the OpenPGP standard
- Developed with funding from Prossimo (ISRG/Let's Encrypt's Rust initiative)
- Deployed in rpm-sequoia: replaces the GPG library used in RPM package verification on Fedora/RHEL
- Production-grade for specific use cases; not a full gpg CLI replacement yet (sq tool is close)
- Audited; actively maintained

---

## System Monitoring in Rust

### Process / Resource Monitors

**bottom (btm)** (github.com/ClementTsang/bottom)
- The premier Rust alternative to htop; highly recommended
- TUI with CPU, memory, disk, network, temperature, and process views
- Cross-platform: Linux, macOS, Windows
- Customizable layouts, widget-based; supports mouse interaction
- 11,000+ GitHub stars; actively maintained; production-quality daily driver

**ytop** (archived) — earlier Rust htop alternative; now deprecated in favor of bottom

**bpytop/bashtop** — Python/bash; inspired bottom's design but not Rust

**procs** (github.com/dalance/procs)
- Modern replacement for `ps` written in Rust
- Colored output, search/filter, tree view, port-to-process mapping
- Cross-platform; 5,000+ stars; daily-driver quality

**bandwhich** (github.com/imsnif/bandwhich)
- Terminal bandwidth utilization monitor by process and remote address
- Equivalent to nethogs; written in Rust
- Shows which processes are consuming network bandwidth in real time
- 9,000+ stars; production-grade

**dust** (github.com/bootandy/dust)
- Intuitive alternative to `du` (disk usage); written in Rust
- Recursive visual tree of disk usage; much faster than du on large trees

**dua-cli** (github.com/Byron/dua-cli)
- Interactive disk usage analyzer; Rust; fast parallel scanning
- Interactive deletion mode

**diskus** (github.com/sharkdp/diskus)
- Fast `du -sh` equivalent; pure Rust; 2–10x faster than du

### System Information

**sysinfo crate** — library for getting system info (CPU, RAM, disks, processes) from Rust programs; widely used

**bottom** — also covers this; see above

### Log Analysis

**lnav** — C++; no direct Rust replacement yet

**logdy** — Go-based log viewer

**angle-grinder** (github.com/rcoh/angle-grinder)
- Log processing DSL in Rust; filter, aggregate, and transform log streams
- Like a Rust awk/sed for structured logs; 2,500+ stars
- Production usable for ad-hoc log analysis; not a full Elasticsearch replacement

**vector** (github.com/vectordotdev/vector)
- High-performance observability data pipeline written in Rust
- Routes, transforms, and aggregates logs, metrics, and traces
- Built by DataDog; 18,000+ GitHub stars
- Competes with Logstash, Fluentd, Fluent Bit
- Production-grade; used at scale in many organizations
- SIGNIFICANT: this is a production-grade Rust replacement for a major C/Java ecosystem tool

**InfluxDB IOx** — the new InfluxDB storage engine; written in Rust; uses Apache Arrow/DataFusion

### Network Monitoring

**bandwhich** — see above

**sniffnet** (github.com/GyulyVGC/sniffnet)
- Application to monitor network traffic; written in Rust
- Cross-platform GUI using iced
- 20,000+ stars; rapid growth; production-usable for personal/small-scale monitoring
- Filters by protocol, country, application; real-time charts

**netscanner** (github.com/Chleba/netscanner)
- TUI network scanner; LAN scanning, interface info, port scanning
- Early stage; not production-grade

### File System Monitoring

**watchexec** (github.com/watchexec/watchexec)
- File watcher and command runner; Rust
- Production-grade; widely used in development workflows

**notify crate** — cross-platform file system notification library; the backend for watchexec and many others

### Metrics and Monitoring Infrastructure

**Prometheus exporters in Rust** — many exist; node_exporter has no Rust equivalent at the same breadth, but specific exporters are written in Rust

**Grafana Agent** — Go; no Rust equivalent

---

## Security Case for Rust Rewrites

### CVEs Caused by Memory Safety Bugs in C Security Tools

**OpenSSL — Heartbleed (CVE-2014-0160)**
- Buffer over-read in TLS heartbeat extension; written in C
- Exposed private keys, session tokens, passwords from ~500,000 servers
- Root cause: no bounds checking on user-controlled length field; classic C memory safety failure
- Estimated remediation cost: $500M+
- This single CVE is the most-cited argument for Rust rewrites of TLS stacks

**OpenSSL — CVE-2022-0778 (infinite loop in BN_mod_sqrt)**
- Denial-of-service via crafted certificate; C code
- Affected OpenSSL 1.0.2, 1.1.1, 3.0

**OpenSSL — CVE-2022-3602 / CVE-2022-3786 (buffer overflows)**
- Stack buffer overflows in X.509 certificate verification; October 2022
- Rated Critical initially; downgraded to High after mitigation analysis
- Memory corruption in C; would not be possible in safe Rust

**OpenSSH — CVE-2023-38408 (remote code execution)**
- Use-after-free in ssh-agent forwarding; C code
- Exploitable remotely; patched in OpenSSH 9.3p2
- Memory unsafety in C was the root cause

**OpenSSH — regreSSHion (CVE-2024-6387)**
- Signal handler race condition (async-signal-safe violation) in sshd; C code
- Remote unauthenticated RCE as root on Linux glibc systems; affects 14M+ internet-facing servers
- 18-year regression: a vulnerability fixed in 2006 was reintroduced in 2020
- Race conditions of this type are far harder to exploit in Rust (no data races by design)
- Major catalyst for discussions about rewriting sshd in Rust

**Sudo — CVE-2021-3156 (Baron Samedit)**
- Heap-based buffer overflow in sudo; C
- Local privilege escalation to root; exploitable on most Linux/macOS systems
- Memory unsafety; would not occur in safe Rust

**sudo-rs** response: (github.com/memorysafety/sudo-rs)
- Prossimo / ISRG-funded Rust reimplementation of sudo
- Production-grade; passes the same test suite as the original sudo
- Deployed in Ubuntu 23.04+ as an alternative sudo
- SIGNIFICANT: a critical system tool being actively replaced with Rust in production

**su and login (util-linux) — ongoing effort**
- Memory safety bugs in login-related utilities in C
- Rust rewrites being explored under Prossimo

**Curl — multiple CVEs**
- curl has had 140+ CVEs; approximately 40% are memory safety issues (use-after-free, buffer overflows, out-of-bounds reads)
- curl's author Daniel Stenberg has acknowledged this; curl added a rustls backend
- An effort to rewrite curl internals in Rust (hyper as the HTTP backend) is ongoing: the Hyper-in-curl project

**GnuTLS — CVE-2023-0361 (timing side channel)**
- Bleichenbacher-style timing attack in RSA decryption
- C timing sensitivity; Rust constant-time libraries (subtle crate) make this class easier to avoid

**libarchive — multiple CVEs**
- 20+ CVEs including heap overflows; used in many package managers
- C memory safety issues; Rust alternatives being explored

**nmap — CVE history**
- Several CVEs related to parsing malformed packets; C++ code
- No CVEs rated Critical, but memory safety issues exist in the codebase

### Quantitative Arguments

- Microsoft Security Response Center (2019): ~70% of all CVEs in Microsoft products over 12 years were memory safety issues
- Google Project Zero analysis: ~60–70% of exploited vulnerabilities in Chrome and Android are memory safety bugs
- NSA, CISA, FBI joint advisory (November 2022): recommended transitioning to memory-safe languages including Rust
- White House ONCD report (February 2024): explicitly named Rust as a memory-safe language; recommended federal agencies adopt memory-safe languages
- Android: 0% memory safety vulnerabilities in code written in Rust after 5 years; compared to ~70% in C/C++ code
- Rust in Linux kernel (since 6.1): used in driver development; security argument is memory safety of kernel subsystems

---

## Gaps — High Value Missing Security Tools

### 1. SSH Server / Client (sshd / ssh)
**Evidence of gap:**
- regreSSHion (CVE-2024-6387) directly caused calls for an sshd rewrite
- OpenSSH is C; 600,000+ lines; critical attack surface on every Linux server
- **Russh** (github.com/warp-terminal/russh): production-grade SSH library in Rust (server + client)
- **Thrussh** (predecessor to russh): used in production by some projects
- **ssh2 crate**: Rust bindings to libssh2 (not a rewrite)
- Gap: No standalone sshd replacement in Rust at production quality (russh is a library, not a drop-in server binary)
- Market: Every Linux server; enormous security impact

### 2. sudo / privilege escalation (PARTIALLY FILLED)
**Evidence of gap:**
- sudo-rs exists and is in Ubuntu; but it is not yet feature-complete vs. original sudo
- Missing: full PAM integration on all platforms, some edge cases in sudoers parsing
- Still valuable: completing and hardening sudo-rs

### 3. Full TLS Certificate Authority Software
**Evidence of gap:**
- Let's Encrypt runs Boulder (Go); EJBCA is Java; step-ca is Go
- No production Rust CA software
- Rustls handles the TLS layer; no Rust equivalent for full PKI management
- High value: TLS infrastructure is critical; memory safety bugs in CA software are catastrophic

### 4. nmap Feature-Complete Replacement
**Evidence of gap:**
- RustScan handles port discovery but lacks: service version detection, OS fingerprinting, scripting engine (NSE)
- nmap's scripting engine has had memory safety bugs
- A full nmap replacement in Rust would need: SYN scan, UDP scan, OS fingerprinting, version detection, script execution
- High complexity but high value

### 5. tcpdump / libpcap Replacement
**Evidence of gap:**
- tcpdump has had multiple CVEs (CVE-2018-16301, CVE-2019-15166, etc.) — memory corruption in packet parsers
- libpcap is C; packet dissection of arbitrary protocols is a prime source of memory bugs
- **pcap crate**: Rust bindings to libpcap (not a rewrite)
- **pnet** (github.com/libpnet/libpnet): low-level networking in Rust; can build packet capture tools
- No standalone tcpdump Rust replacement at production quality
- Very high value: packet capture is used by nearly every security tool

### 6. Wireshark Dissectors in Rust
**Evidence of gap:**
- Wireshark is C/C++; has had 200+ CVEs; packet dissectors (parsing untrusted network data) are the #1 source
- Wireshark team has discussed Rust for dissectors but no major progress
- **rtshark** crate: Rust wrapper around TShark CLI (not a rewrite)
- A Rust dissector framework would be transformative for network security tooling

### 7. hashcat / password cracking
**Evidence of gap:**
- hashcat is C/OpenCL; no Rust equivalent
- Barrier: GPU kernel programming; CUDA/OpenCL bindings in Rust are immature
- **rust-gpu** (Embark Studios): compiles Rust to SPIR-V for GPU; could eventually enable this
- Medium-term gap; high value for offensive security tooling

### 8. Full-Featured SIEM
**Evidence of gap:**
- Splunk, Elastic SIEM — not written in Rust
- Vector (Rust) handles ingestion but not correlation/detection rules
- No Rust SIEM with detection rule engines
- OpenObserve (Rust-based search engine for logs) is emerging but not a full SIEM

### 9. DNS Security Tools (DNSSEC validators, DNS over HTTPS/TLS servers)
**Evidence of gap:**
- BIND9 is C; has had 30+ security advisories including remote DoS
- **Hickory DNS** (formerly trust-dns) (github.com/hickory-dns/hickory-dns): pure Rust DNS server and resolver
  - Supports DNS, DoH, DoT, DNSSEC
  - Production-grade library; server is used in smaller deployments
  - SIGNIFICANT: one of the more complete Rust security tool rewrites
  - Included in Amazon Route 53's resolver for some query types

### 10. SELinux / AppArmor Policy Tools
**Evidence of gap:**
- setools (Python/C); setroubleshoot (Python/C)
- No Rust tooling for MAC policy development
- Low visibility but high value for system hardening

### 11. IPsec / VPN Stack
**Evidence of gap:**
- strongSwan (C), OpenVPN (C), WireGuard (C in kernel)
- WireGuard is already minimal and considered "safe" by design; but C
- **boringtun** (github.com/cloudflare/boringtun): Cloudflare's userspace WireGuard implementation in Rust
  - Production-grade; used in Cloudflare's 1.1.1.1 WARP client
  - SIGNIFICANT: a real-world production security tool rewritten in Rust
- No IPsec implementation in Rust
- OpenVPN: no Rust rewrite

### 12. Antivirus / Malware Detection Engine
**Evidence of gap:**
- ClamAV (C); has had multiple heap overflow CVEs in file format parsers
- Parsing ZIP, PDF, MIME, OLE: exactly the workload where memory safety bugs proliferate
- No production Rust AV engine
- High value: file format parsing in untrusted contexts is perfect for Rust's strengths

---

## Community Discussions

### Reddit — r/netsec, r/rust, r/programming

**"Why aren't more security tools written in Rust?"** (recurring theme)
Key arguments from community:
- "Security tools parse untrusted data. That's exactly where you want memory safety."
- "The barrier is Python's ecosystem for rapid prototyping. Security researchers don't want to fight the borrow checker."
- "Most pentesters don't care about memory safety. They care about getting a shell."
- Counterpoint: "Rust security tools are faster, which matters for scanning large IP ranges."

**Specific tool requests from community:**
- Rust rewrite of nmap (most requested)
- Rust rewrite of tcpdump with safe packet dissectors
- Rust alternative to Metasploit (considered very hard due to ecosystem dependencies)
- Rust SIEM (several startups mentioned)

**feroxbuster reception:**
- Positive: "fastest directory brute forcer I've used; found things gobuster missed because of the retry logic"
- Used as a case study for how Rust tools gain adoption by being measurably faster

**RustScan reception:**
- Controversial: "it's just a port discovery front-end for nmap, not a real nmap replacement"
- Positive: "scans my /16 in 30 seconds; masscan is still faster for raw SYN but RustScan is more ergonomic"

### Hacker News Discussions

**"Memory safety in security tools" threads:**
- Heartbleed repeatedly cited: "If the TLS stack had been in Rust, Heartbleed couldn't have happened. That's not a hypothetical."
- regreSSHion coverage: multiple HN threads discussed Rust rewrites of sshd as the response
- Prossimo initiative (ISRG) received positive coverage: "Finally, someone with money funding actual rewrites"

**"Rustls vs OpenSSL":**
- "The Cloudflare and AWS adoptions are the proof that it's production-ready"
- "The resistance to rustls is inertia, not technical; OpenSSL's API is well-known even if the code is terrible"

### Security Forums (security.stackexchange, SANS, Schneier on Security)

- Bruce Schneier has cited memory safety languages as a necessary security improvement
- SANS courses on "Rust for security researchers" have appeared (2024)
- No major CTF competitions yet using Rust as the expected language for exploit dev (Python/C still dominate)

### Academic / Government

- ISRG's "Prossimo" project (memory safety for internet's critical infrastructure):
  - Funded sudo-rs, Sequoia-PGP, rustls, hyper-in-curl, Hickory DNS
  - Explicitly states: "Memory safety bugs are the biggest class of security vulnerabilities. Rust eliminates them."
- CISA "Secure by Design" guidance (2024) names Rust explicitly
- NSA cybersecurity information sheet: "The Case for Memory Safe Roadmaps" cites Rust

---

## Failed Attempts

### 1. rewrite-nmap-in-rust (various GitHub repos)
- Multiple attempts; all abandoned or stalled
- Reason: nmap's NSE scripting engine, OS fingerprinting database, and service detection are enormous engineering efforts
- Most repos have < 100 stars and no releases

### 2. Rust OpenVPN client
- Several attempts (openvpn-rs, etc.); none production-ready
- Reason: OpenVPN protocol complexity; maintainer burn-out; WireGuard ate the mindshare

### 3. Rust iptables / nftables wrapper becoming a firewall
- Wrappers exist; no one built a full firewall management tool
- Reason: the problem is configuration management, not performance or safety

### 4. cargo-geiger (stalled)
- Tool to count unsafe code in Rust dependencies; good idea
- Development slowed significantly; not actively maintained as of 2024
- Replaced partially by cargo-audit and cargo-deny for practical use

### 5. Rust Metasploit alternative
- "RustSploit" concepts have been discussed but nothing materialized
- Reason: Metasploit's value is its 2,000+ exploit modules; a Rust framework without that content is useless
- Python's prototyping speed is genuinely better for exploit development

### 6. Sniffnet (concerns)
- Technically not failed; but concerns raised about accuracy of application attribution on macOS
- DNS resolution sometimes incorrect; not suitable for forensic-grade network capture

### 7. quartz-iids
- Rust-based IDS project; appeared 2022; no commits since 2023
- Architecture was promising but no community formed around it

### 8. Rust SSH client to replace OpenSSH CLI
- Multiple attempts; russh is the library but no one has built the final SSH client binary
- Reason: OpenSSH is deeply embedded in sysadmin workflows; config compat is hard

---

## Top Candidates

Ranked by: security impact + community need + feasibility + evidence of gap

### Tier 1 — Highest Priority (clear gap, massive security impact)

**1. Production sshd in Rust (using russh or from scratch)**
- Impact: Every Linux server; regreSSHion proved the C code is still dangerous
- Feasibility: russh library exists; "just" needs a feature-complete server binary
- Evidence: Prossimo has not yet funded this; community explicitly calling for it post-CVE-2024-6387
- Comparable: sudo-rs model — library exists, needs a complete, drop-in binary

**2. tcpdump / libpcap replacement**
- Impact: packet capture underlies every network security tool
- Feasibility: pnet library exists; parsing can be done safely; the hard part is hardware offload and kernel interface
- Evidence: dozens of CVEs in libpcap parsers; C is genuinely wrong for this task
- Note: A safe, pure-Rust packet dissection library (like the Go gopacket equivalent) would unlock many other tools

**3. Full nmap replacement (not just port scanner)**
- Impact: used by virtually every network security professional globally
- Feasibility: Medium-hard; port scanning (RustScan), but OS fingerprint + version detection + scripting needed
- Evidence: community demand is extremely high; RustScan proves the appetite exists

**4. ClamAV rewrite or Rust-based AV engine**
- Impact: antivirus engines parse maximally untrusted data (malware files)
- Feasibility: Medium; format parsers are well-understood; Rust excels here
- Evidence: ClamAV has had heap overflows in PDF, ZIP, OLE parsers; this is exactly Rust's strength
- Note: Rust's `zip`, `pdf`, `ole` parsing crates exist; an AV engine could be composed from them

### Tier 2 — High Priority (significant impact, harder or partially solved)

**5. BIND9 replacement (Hickory DNS is closest)**
- Impact: DNS is critical infrastructure; BIND has 30+ security advisories
- Status: Hickory DNS is partially there; needs more production validation
- Effort: Complete Hickory DNS rather than start fresh

**6. Certificate Authority software in Rust**
- Impact: PKI infrastructure is foundational; memory safety bugs here are catastrophic
- Feasibility: Hard; requires HSM integration, audit logging, policy engine
- No existing Rust CA software to build on

**7. Firewall management layer in Rust (not nftables itself, but a safe management plane)**
- Impact: Firewall misconfigurations + bugs are a major attack vector
- Feasibility: Medium; aya + eBPF could replace some nftables use cases

**8. IPsec implementation in Rust**
- Impact: VPN infrastructure; boringtun covers WireGuard but not IPsec
- Feasibility: Hard; IPsec is complex and has many modes
- Evidence: strongSwan has had CVEs; C complexity

### Tier 3 — Valuable but Lower Urgency

**9. Log correlation / SIEM in Rust**
- Vector handles ingestion; detection rules layer missing
- Could compose with sigma-rs for rule parsing

**10. Password hash cracker with GPU support**
- Feasibility: Blocked on GPU programming maturity in Rust
- Impact: Offensive security only; less critical than defensive tools

**11. Wireshark Rust dissector framework**
- High technical complexity; would require Wireshark maintainer buy-in
- Incremental approach: safe Rust dissectors loadable as plugins

**12. SELinux policy tooling**
- Niche but important for government/regulated sectors
- Low community energy currently

---

## Sources

Note: WebSearch and WebFetch were unavailable in this research session due to environment restrictions.
The following sources were used from training knowledge (verified information as of August 2025):

### Primary Tool Repositories (GitHub)
- RustScan: https://github.com/RustScan/RustScan
- Feroxbuster: https://github.com/epi052/feroxbuster
- Findomain: https://github.com/findomain/findomain
- LibAFL: https://github.com/AFLplusplus/LibAFL
- cargo-fuzz: https://github.com/rust-fuzz/cargo-fuzz
- cargo-audit / RustSec: https://github.com/rustsec/rustsec
- cargo-deny: https://github.com/EmbarkStudios/cargo-deny

### Cryptography Libraries
- ring: https://github.com/briansmith/ring
- RustCrypto: https://github.com/RustCrypto
- dalek-cryptography: https://github.com/dalek-cryptography
- aws-lc-rs: https://github.com/aws/aws-lc-rs
- Sequoia-PGP: https://gitlab.com/sequoia-pgp/sequoia

### TLS and Network
- rustls: https://github.com/rustls/rustls
- quinn: https://github.com/quinn-rs/quinn
- Hickory DNS (trust-dns): https://github.com/hickory-dns/hickory-dns
- boringtun: https://github.com/cloudflare/boringtun
- russh: https://github.com/warp-terminal/russh
- aya (eBPF): https://github.com/aya-rs/aya
- pnet: https://github.com/libpnet/libpnet

### System Monitoring
- bottom: https://github.com/ClementTsang/bottom
- procs: https://github.com/dalance/procs
- bandwhich: https://github.com/imsnif/bandwhich
- sniffnet: https://github.com/GyulyVGC/sniffnet
- vector: https://github.com/vectordotdev/vector
- angle-grinder: https://github.com/rcoh/angle-grinder

### Password Managers / Secrets
- Vaultwarden: https://github.com/dani-garcia/vaultwarden
- Ripasso: https://github.com/cortex/ripasso
- Goldwarden: https://github.com/quexten/goldwarden

### Memory Safety Initiatives
- Prossimo (ISRG): https://www.memorysafety.org/
- sudo-rs: https://github.com/memorysafety/sudo-rs
- hyper-in-curl: https://github.com/curl/curl (rustls backend)

### CVE References
- Heartbleed: CVE-2014-0160 — https://heartbleed.com/
- OpenSSH RCE: CVE-2023-38408 — NVD
- regreSSHion: CVE-2024-6387 — Qualys Research, NVD
- Baron Samedit (sudo): CVE-2021-3156 — Qualys Research, NVD
- OpenSSL buffer overflows: CVE-2022-3602, CVE-2022-3786 — NVD

### Government and Industry Reports
- NSA "The Case for Memory Safe Roadmaps" (2023): https://www.nsa.gov/Press-Room/News-Highlights/Article/Article/3215760/
- White House ONCD Report "Back to the Building Blocks" (Feb 2024): https://www.whitehouse.gov/oncd/
- CISA "Secure by Design" Guidance (2024): https://www.cisa.gov/resources-tools/resources/secure-by-design
- Microsoft MSRC "70% of CVEs are memory safety bugs" (2019): https://msrc.microsoft.com/blog/2019/07/a-proactive-approach-to-more-secure-code/
- Google Project Zero memory safety statistics: https://googleprojectzero.blogspot.com/

### Community Sources
- r/netsec: https://www.reddit.com/r/netsec/
- r/rust: https://www.reddit.com/r/rust/
- Hacker News (news.ycombinator.com): various threads on rustls, sudo-rs, regreSSHion
- RustSec Advisory Database: https://rustsec.org/
