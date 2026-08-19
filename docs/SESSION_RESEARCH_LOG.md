# Session Research Log

> This document tracks research findings and decisions made during development sessions.
> Entries are historical snapshots; they do not override the current ownership
> model. The current production SDK is `conxius-enclave-sdk`, and the Core
> companion is `lib-conxian-core-enclave`.

---

## Session 2026-07-15: SDK Integration & Ecosystem Alignment

### Objective
1. Check if lib-conxian-core is using the latest SDK crate and all its capabilities
2. Expand research to understand full ecosystem alignment
3. Implement all recommendations

### Expanded Research Findings

#### 1. Ecosystem Crate Inventory

**Published Crates (botshelomokoka on crates.io):**
| Crate | Version | Purpose |
|-------|---------|---------|
| `conxius-enclave-sdk` | 2.0.11 | Production Vault SDK |
| Historical SDK alias entry | 2.0.8 | Deprecated; not a current package |
| `conxian-core` | 0.1.4 | Gateway core |
| `conxian_api` | 0.1.4 | HTTP API |
| `conxian_compliance` | 0.1.4 | ZK compliance |
| `conxian_engine` | 0.1.4 | Business logic |

#### 2. Unique Value of lib-conxian-core

**Modules UNIQUE to lib-conxian-core (not in SDK):**
- `control_model/` - Trust tiers, lifecycle states, invariants (HIGH value)
- `anchoring.rs` - State persistence models (HIGH value)
- `adapters/` - Universal chain adapter trait (CXIP-21) (HIGH value)
- `deployment.rs` - Deployment manifests (MEDIUM value)

#### 3. Beta Dependency Watchlist

```
bitcoin = "0.33.0-beta"         # Watch for stable (SDK)
secp256k1 = "0.32.0-beta.2"    # Watch for stable (SDK)
k256 = "0.14.0-rc.9"           # Watch for stable (SDK)
```

#### 4. Cross-Repository Dependencies

```
conxian-ui → conxian-gateway → lib-conxian-core → conxius-enclave-sdk
conxius-wallet → conxius-enclave-sdk → lib-conxian-core
conxian-nexus → lib-conxian-core
```

#### 5. Open Issues Summary

- conxian-gateway: 11 open (P0: TypeScript SDK)
- conxius-wallet: 3 open (P1: tech debt, Silent Payments)
- conxian-nexus: 3 open (infrastructure)
- conxius-enclave-sdk: 0 open (all P1 resolved)

### Key Findings

#### 1. Critical Discovery: Vault SDK Location
**Finding**: The production Vault SDK is **NOT** in this repository. It is published as [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) v2.0.11.

**Evidence**:
- crates.io shows 7 crates published by botshelomokoka
- `conxius-enclave-sdk` v2.0.11 is the production SDK with:
  - 57 Rust source files
  - 348 public API items
  - WASM bindings
  - FROST DKG, Ark, BitVM2, MuSig2, Fedimint, Lightning, 30+ chains

#### 2. Version Mismatch
| Local | Published crates.io | Notes |
|-------|---------------------|-------|
| `lib-conxian-core` v0.2.10 | N/A (not published) | This repo NOT published |
| N/A | `conxian-core` v0.1.4 | Different crate, gateway core |
| N/A | `conxius-enclave-sdk` v2.0.11 | Production Vault SDK |

#### 3. Local vs SDK Implementations

| Local Module | SDK Status | Gap |
|--------------|------------|-----|
| `src/musig2.rs` | Simplified stub | SDK uses real musig2 crate with BIP-327 |
| `src/bitvm2.rs` | Stub | SDK has full BitVm2Orchestrator |
| `src/wallet.rs` | Basic | SDK has hardware attestation |
| `src/sdk_primitive.rs` | Deprecated | Use SDK directly |
| `src/control_model/` | Unique | ✅ No gap |
| `src/protocol/frost.rs` | Parity | ✅ No gap |

#### 4. GitHub Issues Analysis

**conxius-enclave-sdk**: ✅ All P1 issues closed (0 open)

**conxian-gateway** (11 open):
- P0: Publish TypeScript SDK to npm
- P1: RGB Full stash resolver integration
- P1: DLC CET construction path
- P1: BitVM Groth16 verifier boundary
- Research: Babylon Cosmos SDK light client, DLC oracle, Liquid E2E

**conxius-wallet** (3 open):
- P1: Technical debt reduction
- P1: Strict CI/CD baseline
- Feature: Native Silent Payment (BIP-352)

### Actions Taken

1. ✅ Added `conxius-enclave-sdk` v2.0.11 as optional dependency with `enclave` feature
2. ✅ Fixed Lightning version: 0.2.3 → 0.2.4
3. ✅ Updated homepage to conxian-labs.com
4. ✅ Deprecated local VaultSDK (use conxius-enclave-sdk directly)
5. ✅ Updated AGENTS.md with correct SDK guidance
6. ✅ Updated README.md with crate relationship table
7. ✅ Updated PRD.md with crate relationships
8. ✅ Updated GAP_ANALYSIS_AND_SCORING.md with findings

### Code Changes
- `Cargo.toml`: Added SDK dependency, fixed versions, updated metadata
- `src/lib.rs`: Added SDK re-exports, deprecated local VaultSDK
- `AGENTS.md`: Clarified SDK location
- `README.md`: Added crate relationship table
- `docs/PRD.md`: Added crate relationships
- `docs/GAP_ANALYSIS_AND_SCORING.md`: Comprehensive update

### Commit
```
cf8133f refactor: clarify Vault SDK location and integrate conxius-enclave-sdk
```

### Recommendations

#### Short-term (This Sprint)
1. Mark `src/sdk_primitive.rs` as fully deprecated
2. Document migration path from local VaultSDK to conxius-enclave-sdk
3. Update conxian-gateway issues tracking

#### Medium-term (Next Quarter)
1. Remove duplicated implementations (musig2, bitvm2) from lib-conxian-core
2. Keep only unique protocol primitives (control_model, anchoring, adapters)
3. Add integration tests between lib-conxian-core and conxius-enclave-sdk

#### Long-term (Strategic)
1. Decide: Merge lib-conxian-core into conxius-enclave-sdk?
2. Or: Keep lib-conxian-core as thin protocol types wrapper?
3. Consider publishing lib-conxian-core to crates.io for broader use

### Dependencies to Watch
- `bitcoin = "0.33.0-beta"` → stable release
- `secp256k1 = "0.32.0-beta.2"` → stable release
- `k256 = "0.14.0-rc.9"` → stable release

---

## Previous Sessions

*Add new sessions above this line*

---

## Session 2026-08-18: Ecosystem-Wide Audit, Gap Mapping & v0.3.2 Research Update

### Objective
1. Perform comprehensive audit and gap mapping of all repository code, documentation, and external SDK/crate references.
2. Reconcile `conxius-enclave-sdk` version references across `Cargo.toml` (pinned `v2.0.16`), `docs/GAP_ANALYSIS_AND_SCORING.md`, and `docs/SESSION_RESEARCH_LOG.md`.
3. Update Candidate Matrix and protocol readiness scores to reflect current v0.3.2 fail-closed boundaries and SDK-owned signing capabilities.
4. Synchronize ecosystem roadmap, governance scorecards, and verification script results.

### Expanded Research Audit Findings

#### 1. Vault SDK Boundary Alignment (`conxius-enclave-sdk` v2.0.16)
- **Current Core Integration**: `Cargo.toml` pins `conxius-enclave-sdk` to Git tag `v2.0.16` (manifest version `2.0.16`) with optional feature gates (`enclave`, `sdk-blockchain`, `sdk-cross-cutting`, `sdk-rails`, `sdk-nexus`, `sdk-infrastructure`, `sdk-signing`, `full-sdk`).
- **Ownership Separation**: Core (`lib-conxian-core`) remains zero-secret-egress, fail-closed for signing and attestation verification. Production cryptographic operations (MuSig2 session aggregation, FROST DKG, hardware enclave attestation, BitVM2 execution) are owned by `conxius-enclave-sdk`.

#### 2. Protocol Primitives & Fail-Closed Boundaries
- **FROST & MuSig2**: Core provides typed structures and fail-closed placeholders; production signing and session aggregation are delegated to `conxius-enclave-sdk`.
- **BIP-322**: Core enforces strict input message and transaction structure parsing; signature validation and script satisfaction remain downstream.
- **DLC & RGB**: Core maintains equation verification and intent-bound policy validation; execution, funding, CETs, and stash resolution are fail-closed boundaries.
- **Universal Adapters (CXIP-21)**: Core provides structured adapter DTOs for Bitcoin, EVM, Cosmos, Solana, Move, and Substrate; light-client verification resides in downstream services.

#### 3. Scoring Matrix Re-calibration (v0.3.2)
- Strategic Alignment (40%), Technical Readiness (30%), Ecosystem Demand (30%).
- Updated Candidate Matrix entries reflect `conxius-enclave-sdk` v2.0.16 capabilities and core protocol stability.

### Actions Executed
- Updated `docs/GAP_ANALYSIS_AND_SCORING.md` to reflect `conxius-enclave-sdk` v2.0.16 alignment and current protocol status.
- Updated `docs/PHASE1_ISSUES_ROADMAP.md` with revised technical debt and protocol gap metrics.
- Verified zero architectural contamination via `python3 scripts/verify_contamination_guard.py`.
- Verified tracked file hygiene via `python3 scripts/verify_tracked_artifacts.py`.
- Verified release hygiene via `python3 scripts/verify_release_hygiene.py`.
- Verified full Python test suite (`python3 -m unittest discover -s scripts/tests -p 'test_*.py'`).
- Executed core Rust test suite (`cargo test`).


---

## Session 2026-08-19: RGB Adapter Guard Hardening & Governance Synthesis

### Objective
1. Perform research synthesis across all repository knowledge bases, open branches, and gap scorecards.
2. Hardened fail-closed contract ID validation in `RGBStockAdapter` (`src/rgb/mod.rs`).
3. Expand research documentation across `GAP_ANALYSIS_AND_SCORING.md`, `SESSION_RESEARCH_LOG.md`, and `UNIVERSAL_SUPPORT_RESEARCH.md`.

### Expanded Findings & Execution
- **RGB Adapter Contract Lookup**: Hardened `RGBStockAdapter::get_contract_details` to return `RGBError::InvalidContractId` on empty or whitespace inputs, ensuring consistency with `RGBSkeletonAdapter`.
- **Governance Alignment**: Updated candidate scores for BitVMX, BitVM3, and ZKCP in `docs/UNIVERSAL_SUPPORT_RESEARCH.md`.
- **System Verification**: Verified core unit tests, scripts, contamination guard, and tracked artifact compliance.
