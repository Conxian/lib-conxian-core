# Session Research Log

> This document tracks research findings and decisions made during development sessions.

---

## Session 2026-07-15: SDK Integration & Remediation

### Objective
Check if lib-conxian-core is using the latest SDK crate and all its capabilities. Remediate issues and align implementations.

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
