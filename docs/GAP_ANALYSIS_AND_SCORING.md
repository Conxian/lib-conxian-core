# Gap Analysis & Implementation Scoring (CON-1305)

This document maps identified protocol gaps, code quality issues, CI/CD gaps, and architectural concerns to research status and implementation priority scoring. **Last comprehensive review: 2026-06-28 (Jules agent session).**

## Scoring Rubric
- **Strategic Alignment (40%)**: Core sovereignty, Bitcoin-native, Vault SDK boundary.
- **Technical Readiness (30%)**: Specification stability, dependency availability, implementation maturity.
- **Ecosystem Demand (30%)**: Partner requirements, TVL potential, security criticality.

---

## A. Protocol Feature Candidates

| Candidate | Strategic | Readiness | Demand | Total | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **MuSig2 Aggregation (G-10)** | 40 | 30 | 30 | **100** | **Stub — needs real impl** |
| **DLC Primitives (G-06)** | 35 | 25 | 30 | **90** | **Stub** |
| **Babylon Staking (G-43)** | 35 | 25 | 30 | **90** | **Implemented** |
| **BitVM2 Multi-Party (G-11)**| 40 | 30 | 20 | **90** | **Skeletal (synthetic segments)** |
| **BIP-322 (G-09)** | 40 | 30 | 20 | **90** | **Implemented** |
| **BitVMX (G-44)** | 40 | 15 | 30 | **85** | Researching |
| **BitVM3 (G-20)** | 40 | 10 | 30 | **80** | Directional |
| **ZKCP (G-50)** | 35 | 15 | 20 | **70** | Researching |
| **FROST Threshold Sig (G-14)** | 35 | 20 | 25 | **80** | **Stub (dummy sigs)** |
| **ERC-7683 Intent Solving (G-12)** | 25 | 15 | 25 | **65** | **Stub (standard still Draft)** |
| **OP_CAT Covenants (G-15)** | 30 | 10 | 20 | **60** | **Stub (BIP-347 not activated)** |
| **Fedimint (G-16)** | 30 | 20 | 20 | **70** | **Empty module (23 lines)** |

---

## B. CI/CD & Automation Gaps

| # | Gap | Severity | Score | Status |
| :--- | :--- | :--- | :--- | :--- |
| **CI-01** | No MSRV verification job (CI only tests latest stable) | **HIGH** | 90 | Open |
| **CI-02** | No `cargo doc` check in CI (doc warnings go undetected) | MEDIUM | 70 | Open |
| **CI-03** | No code coverage tooling (tarpaulin/llvm-cov) | MEDIUM | 65 | Open |
| **CI-04** | No benchmark infrastructure (`cargo bench`) | LOW | 40 | Open |
| **CI-05** | `dependency-review.yml` depends on external `Conxian/.github` org workflow — single point of failure | MEDIUM | 60 | Open |
| **CI-06** | No Windows/macOS CI matrix (only ubuntu-latest) — acceptable for no_std-ish lib but worth documenting | LOW | 35 | Open |
| **CI-07** | No `cargo deny` (license compliance + duplicate dep detection) | MEDIUM | 60 | Open |

---

## C. Code Quality & Architecture Gaps

| # | Gap | Severity | Score | Status |
| :--- | :--- | :--- | :--- | :--- |
| **CQ-01** | `src/control_model.rs` is monolithic (1,131 lines) — should be split into sub-modules (trust, lifecycle, control_plane) | MEDIUM | 65 | Open |
| **CQ-02** | Heavy stub prevalence: crypto (WitnessEncryption, AdaptorSignature, PVDE all `NotImplemented`), lightning (Bolt12 fails closed), FROST (dummy signatures), MuSig2 partial sig aggregation (dummy) | **HIGH** | 85 | Open |
| **CQ-03** | `src/enclave/mod.rs` — 23 lines, all pass-through stub | MEDIUM | 60 | Open |
| **CQ-04** | `src/fedimint/mod.rs` — 23 lines, near-empty module | LOW | 45 | Open |
| **CQ-05** | `src/cjcs.rs` — 18 lines, placeholder | LOW | 35 | Open |
| **CQ-06** | Missing `examples/` directory — no API usage examples for integrators | MEDIUM | 65 | Open |
| **CQ-07** | Missing `benches/` directory — no performance regression detection | LOW | 40 | Open |
| **CQ-08** | Missing `rust-toolchain.toml` — toolchain not pinned for reproducibility | MEDIUM | 70 | Open |
| **CQ-09** | MSRV declared as 1.82 in Cargo.toml but not enforced in CI | **HIGH** | 85 | Open |
| **CQ-10** | `SovereignHandshake` (UX/display concept) lives in `src/wallet.rs` — should be in Gateway, not core | LOW | 40 | Open |
| **CQ-11** | `wasm-bindgen` + `getrandom/js` features suggest WASM target support, contradicting "no environment-specific branching" architectural boundary | MEDIUM | 55 | Open |
| **CQ-12** | `rgb-std = "0.12.0-rc.3"` — Release Candidate dependency in stable library | MEDIUM | 55 | Open |

---

## D. Test Coverage Gaps

| # | Gap | Severity | Score | Status |
| :--- | :--- | :--- | :--- | :--- |
| **TST-01** | `src/anchoring.rs` (284 lines) — **zero tests** | **HIGH** | 85 | Open |
| **TST-02** | `src/contract_bridge.rs` (69 lines) — **zero tests** | MEDIUM | 65 | Open |
| **TST-03** | `src/cjcs.rs` (18 lines) — **zero tests** | LOW | 30 | Open |
| **TST-04** | 0 doc-tests (no API usage verification in docs) | MEDIUM | 60 | Open |
| **TST-05** | No fuzz testing (libfuzzer/cargo-fuzz) for parsing/serialization paths | MEDIUM | 70 | Open |
| **TST-06** | No mutation testing (cargo-mutants) for test quality verification | LOW | 50 | Open |
| **TST-07** | No formal verification (Kani) for critical invariant paths | LOW | 55 | Open |
| **TST-08** | Most tests are happy-path only; error path coverage is light (e.g., BitVM2 error types tested indirectly) | MEDIUM | 60 | Open |

---

## E. Security & Supply Chain Gaps

| # | Gap | Severity | Score | Status |
| :--- | :--- | :--- | :--- | :--- |
| **SEC-01** | No GPG-encrypted security contact in SECURITY.md | MEDIUM | 60 | Open |
| **SEC-02** | No external security audit (only internal self-audit in docs) | **HIGH** | 80 | Open |
| **SEC-03** | No `cargo-deny` configuration for license compliance | MEDIUM | 55 | Open |
| **SEC-04** | `rgb-std` RC dependency may contain unvetted code | MEDIUM | 55 | Open |
| **SEC-05** | No SBOM generation in CI (e.g., `cargo cyclonedx`) | LOW | 35 | Open |

---

## F. Resolved / Fixed

### Session 2 (2026-06-28) — P0 Critical Fixes

| # | Gap | Resolution |
| :--- | :--- | :--- |
| **FIX-03** | CI-01/CQ-09: No MSRV verification in CI | Added `msrv` job to `.github/workflows/main.yml` with `toolchain: "1.82"` |
| **FIX-04** | CQ-02: MuSig2 partial signature aggregation returned dummy bytes | Replaced with real `k256::Scalar` arithmetic via `PrimeField::from_repr`. BIP-327 scalar addition modulo curve order. 10 tests pass. |
| **FIX-05** | TST-01: `anchoring.rs` had zero tests | Added 29 tests: AnchoringTarget, AnchoringRequest (defaults, roundtrip, normalize, idempotency), AnchoringPublication, AnchoringReceipt, AnchoringError (retryable, code, Display, tagged serde), TablelandAnchoringPublisher, OnChainAnchoringPublisher, compact_state_root |

### Session 1 (2026-06-28)

| # | Gap | Resolution |
| :--- | :--- | :--- |
| **FIX-01** | `is_multiple_of()` requires Rust 1.87 but MSRV is 1.82 | Changed to `trimmed.len() % 2 == 0` in `src/bitvm2.rs` |
| **FIX-02** | Cargo.lock out of sync | Ran `cargo generate-lockfile` |

### Health Dashboard (2026-06-28 Session 2)

| Metric | Before | After |
| :--- | :--- | :--- |
| **Tests** | 68 | **102** (+34) |
| **CI Jobs** | 1 (stable) | **2** (stable + MSRV 1.82) |
| **anchoring.rs tests** | 0 | 29 |
| **MuSig2 aggregation** | Dummy bytes | Real k256 scalar arithmetic |

---

## G. Gap Identification & Resolution History
1. **Universal Chain Adapters**: Skeletal implementation complete for Cosmos, Solana, Move, and Substrate (CXIP-21).
2. **BitVM2 Multi-Party**: Resolved (CON-1306). Implemented MuSig2-based Taproot tree aggregation.
3. **BIP-322**: Resolved (CON-1266). Hardened universal message signing logic.
4. **ZKCP**: Scaffolding exists. Research expanded to core library requirements (CON-1313).
5. **MuSig2 Signature Aggregation**: **RESOLVED** — real k256 scalar arithmetic (Session 2).
6. **DLC Primitives**: Scaffolding initiated — still stub (basic intent creation, no oracle verification).
7. **MSRV CI**: **RESOLVED** — added `msrv` job to main.yml (Session 2).
8. **anchoring.rs tests**: **RESOLVED** — 29 comprehensive tests added (Session 2).

## H. Priority Action Plan

### P0 — RESOLVED (Session 1-2, 2026-06-28)
1. ~~**Add MSRV CI job**~~ — ✅ `msrv` job in main.yml (`12c380d`)
2. ~~**Real MuSig2 partial signature aggregation**~~ — ✅ k256 scalar arithmetic (`12c380d`)
3. ~~**Add tests for anchoring.rs**~~ — ✅ 29 tests (`12c380d`)

### P1 — In Progress / Tracked
4. **External security audit scoping** → [#148](https://github.com/Conxian/lib-conxian-core/issues/148)
5. **Split control_model.rs** → [#146](https://github.com/Conxian/lib-conxian-core/issues/146)
6. ~~**Add cargo doc to CI + doc-tests**~~ — ✅ `docs` job + 4 doc-tests + 0 warnings (`f279240`)

### P2 — Partially Resolved / Tracked
7. ~~**Add rust-toolchain.toml**~~ — ✅ pinned stable + components (`f279240`)
8. **Add fuzz testing** → [#147](https://github.com/Conxian/lib-conxian-core/issues/147)
9. ~~**Add cargo deny**~~ — ✅ deny.toml + CI job (`f279240`)
10. ~~**Add examples/ directory**~~ — ✅ vault_sdk_basic.rs (`f279240`)

### P3 — Backlog
11. Resolve rgb-std RC dependency (CQ-12)
12. Audit WASM dependencies vs. architectural boundaries (CQ-11)
13. Add benchmark infrastructure (CI-04)
14. Real FROST implementation → [#145](https://github.com/Conxian/lib-conxian-core/issues/145)

## I. Current Session Health Check (2026-06-28)

| Metric | Value | Status |
| :--- | :--- | :--- |
| **Tests** | 68/68 passing (100%) | 🟢 |
| **Clippy** | Clean (-D warnings) | 🟢 |
| **Rustfmt** | Clean | 🟢 |
| **Build** | Compiles cleanly | 🟢 |
| **Open Issues** | 1 (#143 - Conclave SDK research) | 🟡 |
| **Open PRs** | 1 (#143 - same) | 🟡 |
| **Active Branches** | 20 remote branches (stale cleanup candidate) | 🟡 |
| **Doc Tests** | 0 (none exist) | 🔴 |
| **Fuzz Tests** | 0 | 🔴 |
