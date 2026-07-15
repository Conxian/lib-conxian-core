# Conxian Ecosystem Alignment Report

> **Status**: Active | **Last Updated**: 2026-07-15 | **Version**: 0.2.11

## Executive Summary

This document provides a comprehensive analysis of the Conxian ecosystem crate relationships, identifies unique value propositions for each crate, and outlines a strategic alignment plan for the Conxian Labs organization.

## Ecosystem Crate Inventory

### Published Crates (crates.io)

| Crate | Version | Owner | Purpose | Downloads | Status |
|-------|---------|-------|---------|-----------|--------|
| `conxius-enclave-sdk` | 2.0.11 | botshelomokoka | **Production Vault SDK** | 10 | ✅ Production |
| `lib-conclave-sdk` | 2.0.8 | botshelomokoka | Alias | 10 | ⚠️ Deprecated |
| `conxian-core` | 0.1.4 | botshelomokoka | Gateway core | 23 | ⚠️ WIP |
| `conxian_api` | 0.1.4 | botshelomokoka | HTTP API | 10 | ⚠️ WIP |
| `conxian_compliance` | 0.1.4 | botshelomokoka | ZK compliance | 13 | ⚠️ WIP |
| `conxian_engine` | 0.1.4 | botshelomokoka | Business logic | 13 | ⚠️ WIP |
| `anya-core` | 1.2.0 | botshelomokoka | Bitcoin infra | 556 | ⚠️ Legacy |

### Repository Map

```
Conxian GitHub Organization
├── Conxian/
│   └── Conxian (protocol/DAO-facing)
├── lib-conxian-core (THIS REPO)
│   └── Protocol primitives (shared)
├── conxius-enclave-sdk
│   └── Production Vault SDK
├── conxian-gateway
│   └── Runtime orchestration
├── conxian-nexus
│   └── Observation/verification
├── conxius-wallet
│   └── Sovereign wallet
├── conxian_ui
│   └── Public interface
├── conxius-platform
│   └── Environment scaffolding
├── conxius-orbit
│   └── CLI deployment
└── conxian-labs-site
    └── Labs website
```

## Unique Value Analysis

### lib-conxian-core: What Stays Here

This repository provides **shared protocol primitives** that are consumed by all other ecosystem components. These types define the common language for the Conxian stack.

#### ✅ UNIQUE Modules (Do NOT exist in conxius-enclave-sdk)

| Module | Purpose | Lines | Strategic Value |
|--------|---------|-------|-----------------|
| `control_model/` | Trust tiers, lifecycle states, invariant validation | ~300 | **HIGH** - Core taxonomy |
| `anchoring.rs` | State root persistence models | ~285 | **HIGH** - Protocol contract |
| `adapters/` | Universal chain adapter trait | ~366 | **HIGH** - CXIP-21 |
| `deployment.rs` | Deployment manifests and verification | ~200 | **MEDIUM** - Platform contract |
| `cjcs.rs` | CJC protocol types | ~15 | **LOW** - Legacy |
| `contract_bridge.rs` | Clarity contract interfaces | ~75 | **MEDIUM** - Stacks integration |

#### ⚠️ DEPRECATED Modules (Duplicated in SDK)

| Module | SDK Equivalent | Action |
|--------|----------------|--------|
| `sdk_primitive.rs` | `conxius-enclave-sdk` | Deprecated, use SDK |
| `musig2.rs` | `protocol::musig2` | Deprecated, use SDK |
| `bitvm2.rs` | `protocol::bitvm2` | Deprecated, use SDK |
| `wallet.rs` | `k256` or SDK | Deprecated, use SDK |

#### 🔄 Shared Modules (Partial Overlap)

| Module | lib-conxian-core | conxius-enclave-sdk | Strategy |
|--------|-------------------|---------------------|----------|
| `protocol/frost.rs` | Skeletal | Full implementation | Keep local, reference SDK |
| `enclave/` | X.509 stubs | Full attestation | Keep local stubs only |
| `bitcoin/bip322.rs` | Basic | Full BIP-322 | Keep local |
| `fedimint/` | Basic adapter | Full adapter | Keep local |
| `lightning/` | Basic adapter | Full LND | Keep local |

### conxius-enclave-sdk: Production Vault SDK

This is the **production SDK** for hardware-backed signing and security primitives.

#### Key Capabilities

| Feature | Status | WASM |
|---------|--------|------|
| Hardware Attestation | ✅ Production | ✅ |
| FROST DKG | ✅ v2.0.11 | ✅ |
| Fedimint | ✅ v2.0.7 | ✅ |
| Ark | ✅ v2.0.7 | ✅ |
| BitVM2 | ✅ Production | ✅ |
| MuSig2 | ✅ BIP-327 | ✅ |
| 30+ Chains | ✅ | ✅ |
| Settlement Rails | ✅ | ⚠️ Partial |

#### Dependencies (Beta Warnings)

```
bitcoin = "0.33.0-beta"        # ⚠️ Watch for stable
secp256k1 = "0.32.0-beta.2"    # ⚠️ Watch for stable
k256 = "0.14.0-rc.9"           # ⚠️ Watch for stable
```

## Control Model Deep Dive (CON-791)

The `control_model` module is the **crown jewel** of lib-conxian-core. It defines the canonical trust taxonomy.

### Trust Tier Hierarchy

```rust
pub enum TrustTier {
    Strict,       // T1 - Light client verification required
    Managed,      // T2 - External quorum acceptable
    Expedient,    // T3 - Minimum viable trust
    ObserverOnly, // T4 - No production use
}
```

### Chain Family Support

```rust
pub enum ChainFamily {
    BitcoinUtxo,  // Bitcoin, Liquid, Rootstock
    Evm,         // Ethereum, Base, Arbitrum, Optimism, Polygon
    CosmosIbc,   // CosmosHub, Osmosis, Celestia
    SolanaSvm,   // Solana, Eclipse
    Move,        // Aptos, Sui
    Substrate,   // Polkadot, Kusama
}
```

### Lifecycle States

- `ProtectedActionLifecycleState`: Draft → PendingAuthorization → Timelocked → ReadyForExecution → Executed
- `BitcoinTxLifecycleState`: Draft → Signed → BroadcastPending → InMempool → Confirmed → Finalized

## Cross-Repository Dependencies

### Dependency Graph

```
conxian-ui
    └── conxian-gateway (HTTP API)
            ├── conxian-core (types)
            ├── conxian-engine (business logic)
            ├── conxian-compliance (ZK verification)
            └── lib-conxian-core (shared primitives)
                    └── conxius-enclave-sdk (optional: enclave features)

conxius-wallet
    └── conxius-enclave-sdk (signing, attestation)
            └── lib-conxian-core (control_model types)

conxian-nexus
    └── lib-conxian-core (primitives)
```

### Dependency Matrix

| Consumer | lib-conxian-core | conxius-enclave-sdk | Notes |
|----------|-------------------|---------------------|-------|
| conxian-gateway | ✅ Direct | ⚠️ Optional | Runtime uses both |
| conxius-wallet | ✅ Types only | ✅ Direct | SDK for signing |
| conxian-nexus | ✅ Direct | ❌ | Observing primitives |
| conxian-ui | ❌ | ❌ | Frontend only |

## Strategic Recommendations

### 1. Crate Consolidation (Short-term)

#### Action: Deprecate `lib-conclave-sdk`

- `lib-conclave-sdk` v2.0.8 is an alias for `conxius-enclave-sdk`
- Keep only `conxius-enclave-sdk` as the single SDK crate
- Update documentation to reference the single SDK crate

#### Action: Align `conxian-*` crates

| Current | Recommended | Action |
|---------|-------------|--------|
| `conxian-core` | `conxian-types` | Rename for clarity |
| `conxian_api` | Merge into `conxian-gateway` | Consolidate |
| `conxian_engine` | Merge into `conxian-gateway` | Consolidate |
| `conxian_compliance` | Keep separate | Module of gateway |

### 2. Ownership Transfer (Medium-term)

#### Action: Transfer to Organization

All crates should be owned by a **Conxian Labs** crates.io organization:

```bash
# Create organization: https://crates.io/manage/organizations
# Transfer ownership:
cargo owner --add github:Conxian-Labs:owners <crate-name>
```

**Crates to transfer:**
- conxius-enclave-sdk
- conxian-core
- conxian_api
- conxian_compliance
- conxian_engine
- lib-conclave-sdk (then deprecate)
- anya-core (consider archiving)

### 3. Code Deduplication (Medium-term)

#### lib-conxian-core: Keep Only Unique Code

After deprecation removal in v0.3.0:

**Keep:**
- `control_model/` - Trust tiers, lifecycle states, invariants
- `anchoring.rs` - State persistence types
- `adapters/` - Universal chain adapter trait
- `deployment.rs` - Manifests and verification
- `contract_bridge.rs` - Clarity interfaces
- `cjcs.rs` - Protocol types

**Remove (duplicated in SDK):**
- `sdk_primitive.rs` - Use SDK
- `musig2.rs` - Use SDK
- `bitvm2.rs` - Use SDK
- `wallet.rs` - Use k256

**Consider merging into SDK:**
- `bitcoin/` - Partial overlap with SDK
- `fedimint/` - SDK has better implementation
- `lightning/` - SDK has better implementation
- `protocol/` - Most are stubs

### 4. Version Alignment (Ongoing)

| Crate | Current | Target | Blocker |
|-------|---------|--------|---------|
| lib-conxian-core | 0.2.11 | 0.3.0 | ✅ Migration complete (v0.2.11) |
| conxius-enclave-sdk | 2.0.11 | 2.1.0 | WASM completeness |
| conxian-core | 0.1.4 | 0.2.0 | Feature freeze |

## Open Issues Summary

### conxian-gateway (11 open)

| Priority | Issue | SDK Relevance |
|----------|-------|---------------|
| P0 | Publish TypeScript SDK to npm | **HIGH** - User-facing |
| P1 | RGB Full stash resolver integration | MEDIUM |
| P1 | DLC CET construction path | MEDIUM |
| P1 | BitVM Groth16 verifier boundary | LOW |

### conxius-wallet (3 open)

| Priority | Issue | SDK Relevance |
|----------|-------|---------------|
| P1 | Technical debt reduction | **HIGH** |
| P1 | Strict CI/CD baseline | MEDIUM |
| Feature | Native Silent Payment (BIP-352) | HIGH |

### conxian-nexus (3 open)

| Priority | Issue | SDK Relevance |
|----------|-------|---------------|
| Infrastructure | Auto-merge enablement | LOW |
| Security | Branch protection | MEDIUM |
| Release | Publish GitHub release | MEDIUM |

## Beta Dependency Watchlist

These dependencies are on beta/RC versions and need monitoring:

```toml
# conxius-enclave-sdk
bitcoin = "0.33.0-beta"         # Expected stable: Q4 2026
secp256k1 = "0.32.0-beta.2"    # Expected stable: Q4 2026
k256 = "0.14.0-rc.9"           # Expected stable: Q3 2026

# lib-conxian-core  
bitcoin = "0.32"                # Align with SDK
secp256k1 = "0.31"             # Align with SDK
```

## Action Items

### Immediate (This Sprint)

- [x] Finalize deprecation warnings in lib-conxian-core v0.2.10 ✅ DONE v0.2.11
- [x] Create comprehensive MIGRATION.md ✅ DONE
- [x] Update AGENTS.md with SDK guidance ✅ DONE
- [x] Update PRD.md with crate relationships ✅ DONE

### Short-term (Next Sprint)

- [x] Publish migration guide ✅ DONE (v0.2.11)
- [ ] Deprecate lib-conclave-sdk on crates.io
- [ ] Consider consolidating conxian-* crates

### Medium-term (Next Quarter)

- [ ] Create Conxian Labs crates.io organization
- [ ] Transfer crate ownership to organization
- [ ] Publish lib-conxian-core v0.3.0 with removed deprecated code (completed in v0.2.11)

### Long-term (Strategic)

- [ ] Decide: Keep lib-conxian-core separate OR merge into conxius-enclave-sdk
- [ ] Align all beta dependencies to stable versions
- [ ] Publish comprehensive API documentation

## Contact

- **Conxian Labs**: https://www.conxian-labs.com
- **Support**: support@conxian-labs.com
- **Security**: security@conxian-labs.com
