# Conxian Ecosystem Alignment Report

> **Status**: Active | **Last Updated**: 2026-07-21 | **Version**: 0.3.0

## BIP-110 Position Statement

Conxian Labs **supports BIP-110** (Reduced Data Temporary Softfork) as it aligns with our core principles:

### Why BIP-110 Matters

| Principle | BIP-110 | Conxian |
|----------|---------|---------|
| Peer-to-peer cash | Limits data embedding | Non-custodial architecture |
| Decentralization | Lowers node costs | Hardware-enclave signing |
| Sound money | Focus on monetary use | Trust tier enforcement |
| Original vision | Bitcoin as money | "User owns the train" |

See [docs/BIP110_ALIGNMENT.md](docs/BIP110_ALIGNMENT.md) for full guidance.

## Executive Summary

This document provides a comprehensive analysis of the Conxian ecosystem crate relationships, identifies unique value propositions for each crate, and outlines a strategic alignment plan for the Conxian Labs organization.

## Ecosystem Crate Inventory

### Published Crates (crates.io)

| Crate | Version | Owner | Purpose | Downloads | Status |
|-------|---------|-------|---------|-----------|--------|
| `conxius-enclave-sdk` | 2.0.11 | botshelomokoka | **Production Vault SDK** | 10 | ✅ Production |
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

#### ⚠️ Historical extraction notes (not current Core modules)

| Historical Core module | SDK Equivalent | Current status |
|------------------------|----------------|----------------|
| `sdk_primitive.rs` | `conxius-enclave-sdk` | Removed from current Core API; use the SDK |
| `musig2.rs` | `conxius-enclave-sdk` | Historical extraction record; production sessions are SDK-owned |
| `bitvm2.rs` | `conxius-enclave-sdk` | Historical extraction record; production verification is SDK-owned |
| `wallet.rs` | `k256` or SDK | Historical extraction record; custody/signing are downstream-owned |

#### 🔄 Shared Modules (Partial Overlap)

| Module | lib-conxian-core | conxius-enclave-sdk | Strategy |
|--------|-------------------|---------------------|----------|
| `protocol/frost.rs` | Fail-closed boundary | Full implementation | Keep local boundary, reference SDK |
| `enclave/` | X.509 stubs | Full attestation | Keep local stubs only |
| `bitcoin/bip322.rs` | Strict parser / unsupported boundary | Full BIP-322 | Keep local boundary, reference SDK |
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
    BitcoinUtxo,  // Bitcoin, Lightning
    Statechain,   // Spark, MercuryLayer
    Ark,          // Second, Arkade
    BPoS,         // Babylon, Core, Arch, Midl, Nomic, SideProtocol
    Federation,   // Liquid, Botanix, Bitlayer, Mezo
    MergeMined,   // Rootstock, Fractal
    Anchor,       // Stacks
    Rollup,       // Citrea, Alpen, Alkanes
    AltRollup,    // Bob, Bsquared, Hemi, Corn, Merlin, Rollux, Starknet
    AltLayer1,    // Bevm, Goat
    Csv,          // Rgb
    Hybrid,       // InternetComputer, Flashnet
    Evm,          // Ethereum, Base, Arbitrum, Optimism, Polygon
    CosmosIbc,    // CosmosHub, Osmosis, Celestia
    SolanaSvm,    // Solana, Eclipse
    Move,         // Aptos, Sui
    Substrate,    // Polkadot, Kusama
}
```

The current taxonomy contains exactly 17 families and 48 chain variants, aligned with
[bitcoinlayers.org](https://www.bitcoinlayers.org/) classification. The versioned canonical
risk-profile artifact records an explicit `not_assessed` decision for every entry; it does not
promote the historical example scores in the Phase 1 roadmap. See
[`architecture/RISK_PROFILES.md`](architecture/RISK_PROFILES.md) for the field-level contract,
static-vs-live ownership, and profile review process.

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
                    └── lib-conxian-core-enclave (compatibility adapter)
                            └── conxius-enclave-sdk (optional: enclave features)

conxius-wallet
    └── conxius-enclave-sdk (signing, attestation)
            └── lib-conxian-core (control_model types)

conxian-nexus
    └── lib-conxian-core (primitives)
```

### Dependency Matrix

| Consumer | lib-conxian-core | lib-conxian-core-enclave | conxius-enclave-sdk | Notes |
|----------|-------------------|---------------------------|---------------------|-------|
| conxian-gateway | ✅ Direct | ⚠️ As needed | ⚠️ As needed | Owns runtime orchestration and provider selection |
| conxius-wallet | ✅ Types only | ✅ Boundary | ✅ Direct/through boundary | SDK for signing; adapter maps Core contracts |
| conxian-nexus | ✅ Direct | ❌ | ❌ | Observing and verifying primitives |
| conxian-ui | ❌ | ❌ | ❌ | Frontend only |

## Strategic Recommendations

### 1. Canonical SDK naming (complete)

- Use `conxius-enclave-sdk` as the single current production SDK package and
  repository name.
- Keep `lib-conxian-core-enclave` as the narrow Core/SDK compatibility adapter;
  it is not a second provider or runtime SDK.
- Keep `lib-conxian-core` focused on protocol primitives, invariants, and
  control contracts.

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
- anya-core (consider archiving)

### 3. Code Deduplication (Medium-term)

#### lib-conxian-core: Keep Only Unique Code

The following is the current ownership target after the historical extraction:

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
| lib-conxian-core | 0.3.0 | 0.3.0 | ✅ Current intentional breaking release |
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
- [x] Use `conxius-enclave-sdk` as the canonical production SDK name
- [ ] Consider consolidating conxian-* crates

### Medium-term (Next Quarter)

- [ ] Create Conxian Labs crates.io organization
- [ ] Transfer crate ownership to organization
- [ ] Publish lib-conxian-core v0.3.0 after the release/tag workflow and registry checks complete

### Long-term (Strategic)

- [x] Keep `lib-conxian-core` and `conxius-enclave-sdk` separate with the
  `lib-conxian-core-enclave` compatibility boundary
- [ ] Align all beta dependencies to stable versions
- [ ] Publish comprehensive API documentation
- [ ] Evaluate BIP-110 activation impact across all repos

### BIP-110 Alignment

- [ ] Add `bip110_compliant` feature flag to conxius-enclave-sdk
- [ ] Update fee estimation in conxius-wallet for clean blocks
- [ ] Verify Silent Payments (BIP-352) compatibility with BIP-110
- [ ] Document BIP-110 compliance in all Bitcoin-related code

See [docs/BIP110_ALIGNMENT.md](docs/BIP110_ALIGNMENT.md) for full BIP-110 guidance.

## Contact

- **Conxian Labs**: https://www.conxian-labs.com
- **Support**: support@conxian-labs.com
- **Security**: security@conxian-labs.com
