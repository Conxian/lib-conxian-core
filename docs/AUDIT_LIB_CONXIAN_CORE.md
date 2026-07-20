# Mainnet Readiness Audit: lib-conxian-core (CON-145)

## 1. Executive Summary
**Status:** READY for repository-boundary and fuzz-regression scope; overall mainnet readiness not determined
**Priority Class:** P0
**Last Audit Date:** 2026-04-12

This repository serves as the shared cryptographic and protocol foundation for the Conxian network. The review summarized here covers repository-boundary and fuzz-regression findings; it is not an external cryptographic audit or an overall mainnet-readiness determination.

## 2. Technical Audit Results

### 2.1. Cryptographic Safety
- **BitVM2:** The current core tree no longer contains `src/bitvm2.rs`; production BitVM2 proof verification belongs to `conxius-enclave-sdk`.
- **MuSig2:** The current core tree no longer contains `src/musig2.rs`; production MuSig2 sessions belong to `conxius-enclave-sdk`. The maintained fuzz target covers the upstream `musig2` key-aggregation dependency directly.

### 2.2. Fuzz Regression Coverage
- The current suite has five bounded targets: intent resolution, anchoring receipt deserialization, BIP-322 verification, upstream `bitcoin::psbt::Psbt` deserialization, and upstream `musig2` key aggregation.
- The PSBT and MuSig2 targets are dependency-level parser/aggregation coverage and do not exercise removed core Bitcoin orchestration or Vault APIs.
- `.github/workflows/fuzz-regression.yml` runs the five targets weekly and by manual dispatch, with bounded per-target time and a 2 GiB RSS limit.

### 2.3. Dependency Integrity
- Dependencies are limited to standard, high-assurance crates: `secp256k1`, `sha2`, `ark-works` stack, and `tokio`.
- All transitive dependencies were reviewed for known vulnerabilities (Cargo Audit signal: Clean).

### 2.4. Release Hygiene
- **SemVer:** Enforced version 0.2.0.
- **Changelog:** [`CHANGELOG.md`](../CHANGELOG.md) established.
- **Governance:** `CONTRIBUTING.md`, `SECURITY.md`, and `README.md` updated with mainnet-safety standards.

## 3. Findings & Remediation
| ID | Finding | Severity | Status |
| -- | --- | --- | --- |
| CON-CORE-01 | Production MuSig2 and BitVM2 implementations were duplicated in core. | High | RESOLVED (owned by `conxius-enclave-sdk`) |
| CON-CORE-02 | Lacked explicit mainnet-safety branch policy. | Medium | FIXED (via CONTRIBUTING.md) |
| CON-CORE-03 | Missing versioned changelog. | Low | FIXED |

## 4. Conclusion
The repository-boundary and fuzz-regression findings covered here are **ready for scoped integration checks**. Vault-specific MuSig2 and BitVM2 functionality is owned by `conxius-enclave-sdk`, while this crate's five-target fuzz regression suite covers its current core APIs and direct dependency-level surfaces. This document does not constitute an external cryptographic audit or an overall mainnet-readiness determination.
