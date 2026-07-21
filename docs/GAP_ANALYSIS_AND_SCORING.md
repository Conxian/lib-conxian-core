# Gap Analysis & Implementation Scoring (CON-1305)

> **Session Note**: Updated 2026-07-15 to reflect SDK integration and crate relationship mapping.
>
> **See Also**: [ALIGNMENT.md](./ALIGNMENT.md) for comprehensive ecosystem alignment.

This document maps identified protocol gaps to research status and implementation priority scoring.

## Critical Discovery: SDK Integration (v0.2.10)

**The production Vault SDK is in [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) v2.0.11**, NOT in this repository.

### Crate Relationship Matrix

| Crate | Version | Purpose | Status |
|-------|---------|---------|--------|
| `conxius-enclave-sdk` | 2.0.11 | **Production Vault SDK** - Hardware signing, attestation, FROST DKG, Ark, BitVM2 | ✅ Production |
| `lib-conxian-core` | 0.2.12 | **Protocol primitives** - Types, invariants, chain adapters | ⚠️ Prototype |
| `conxian-gateway` | 0.1.4 | Runtime orchestration and middleware | ⚠️ WIP |

### Local Implementations vs SDK

| Local Module | SDK Module | Gap Status |
|--------------|------------|-------------|
| Removed in-core MuSig2/BitVM2/Vault implementations | `conxius-enclave-sdk` | ✅ **SDK-owned** - production signing, sessions, attestation, and BitVM2 verification live in the SDK |
| `fuzz/fuzz_targets/musig2_aggregate.rs` | upstream `musig2::KeyAggContext` | ✅ **Dependency-level fuzz coverage** |
| `fuzz/fuzz_targets/proof_request_validate.rs` | `src/verifier.rs::ProofVerificationRequest::validate` | ✅ **Structural and contract/policy validation; no cryptographic BitVM2 proof verification** |
| `src/protocol/frost.rs` | `src/protocol/frost.rs` | ✅ Parity |
| `src/control_model/` | N/A | ✅ Unique to lib-conxian-core |

## SDK Capabilities Now Available (via `enclave` feature)

### Core Modules (via conxius-enclave-sdk)

| Module | Capabilities | WASM |
|--------|-------------|------|
| `enclave` | Hardware attestation, StrongBox, Secure Enclave, replay guards | ✅ |
| `protocol/bitcoin` | BIP-322 signing, ECDSA/Schnorr, PSBT | ✅ |
| `protocol/lightning` | LND integration | ⚠️ Missing |
| `protocol/ark` | vTXO tree construction, stateless recovery | ✅ |
| `protocol/bitvm2` | Challenge orchestration | ✅ |
| `protocol/frost` | DKG Round 2 verification | ✅ |
| `protocol/fedimint` | Federation adapter, blinding | ✅ |
| `protocol/musig2` | BIP-327 multi-sig aggregation | ✅ |
| `protocol/settlement` | x402, Wormhole, Boltz, NTT, Bisq | ✅ |
| `protocol/swap_router` | Cross-chain swaps | ⚠️ Missing WASM |
| `protocol/zkml` | Zero-knowledge ML | ⚠️ Missing WASM |

### Beta Dependencies (SDK Warning)

```
bitcoin = "0.33.0-beta"        # ⚠️ Watch for stable
secp256k1 = "0.32.0-beta.2"    # ⚠️ Watch for stable
k256 = "0.14.0-rc.9"           # ⚠️ Watch for stable
```

## Scoring Rubric
- **Strategic Alignment (40%)**: Core sovereignty, Bitcoin-native, Vault SDK boundary.
- **Technical Readiness (30%)**: Specification stability, dependency availability.
- **Ecosystem Demand (30%)**: Partner requirements, TVL potential.

## Canonical risk-profile boundary (CORE-007 / GitHub #177)

The scoring rubric above is a planning and prioritization tool. Its values are **not** canonical
chain-risk scores and must not be copied into `data/risk_profiles/v1.json` or treated as routing
policy. The canonical schema and governance rules live in
[`docs/architecture/RISK_PROFILES.md`](architecture/RISK_PROFILES.md).

The initial schema-v1 profile set records an explicit `not_assessed` decision for every one of the
6 `ChainFamily` values and 23 current `Chain` variants. No profile has an approved score until a
review changes the artifact, profile revision, evidence references, governance reference, focused
tests, documentation, and release notes together. Issue #177 and repository documentation are
change references only; they are not empirical risk evidence.

The six canonical dimensions are unitless strength scores in `0..=100`: data availability,
settlement, bridge, exit mechanism, operator independence/resilience (the normalized
`operator_dependency_score`), and decentralization. A valid zero is distinct from `unknown` and
`not_assessed`. Nexus owns live observations and evidence acquisition; Gateway owns runtime
assessment, persistence, and routing. Core owns only the versioned data contract and invariants.
The Core registry intentionally contains no approved scores until separate governance supplies
public evidence and a versioned artifact revision.

## Candidate Matrix

| Candidate | Strategic | Readiness | Demand | Total Score | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **MuSig2 Aggregation (G-10)** | 40 | 30 | 30 | **100** | **Implemented** (SDK) |
| **FROST Threshold (G-14)** | 40 | 25 | 30 | **95** | **Implemented** (SDK) |
| **DLC Primitives (G-06)** | 35 | 25 | 30 | **90** | **Implemented** (SDK) |
| **Hardware Attestation (G-17)**| 35 | 20 | 30 | **85** | **Implemented** (SDK) |
| **Babylon Staking (G-43)** | 35 | 25 | 30 | **90** | **Implemented** |
| **BitVM2 Multi-Party (G-11)**| 40 | 30 | 20 | **90** | **Implemented** (SDK) |
| **BIP-322 (G-09)** | 40 | 30 | 20 | **90** | **Implemented** (SDK) |
| **Fedimint (G-16)** | 30 | 25 | 25 | **80** | **Implemented** (SDK) |
| **Silent Payments (G-05)** | 35 | 25 | 20 | **80** | **Implemented** (SDK) |
| **RGB Integration (CXIP-20)** | 35 | 20 | 30 | **85** | **Implemented** |
| **Fuzz Testing (CON-1332 / GitHub #147)** | 30 | 30 | 20 | **80** | **Implemented** (4 bounded targets; weekly/manual CI) |
| **BitVMX (G-44)** | 40 | 15 | 30 | **85** | Researching |
| **BitVM3 (G-20)** | 40 | 10 | 30 | **80** | Directional |
| **ZKCP (G-50)** | 35 | 15 | 20 | **70** | Researching |

## Gap Identification & Resolution
1. **Universal Chain Adapters**: Skeletal implementation complete for Cosmos, Solana, Move, and Substrate (CXIP-21).
2. **BitVM2 Multi-Party**: Resolved (CON-1306). Production MuSig2-based Taproot tree aggregation and BitVM2 verification are owned by `conxius-enclave-sdk`; this crate no longer carries the Vault implementation.
3. **BIP-322**: Resolved (CON-1266). Hardened universal message signing logic.
4. **FROST Round 2**: Resolved (CON-1329). Moving from skeletal generation to encrypted share distribution.
5. **Hardware Attestation**: Resolved (CON-1329). Implementing X.509 DER parsing for enclave certificate chains.
6. **MuSig2 Signature Aggregation**: Resolved (G-10). Production signing and session aggregation are owned by `conxius-enclave-sdk`; this crate retains only protocol primitives and direct dependency-level fuzz coverage.
7. **Fedimint**: Resolved (G-16). Transitioning to real cryptographic blinding via `fedimint-client-wasm`.
8. **Silent Payments**: Resolved (G-05). Hardened scanning logic with real ECC point math.
9. **DLC**: Resolved (G-06). Hardened oracle attestation verification.
10. **RGB**: Resolved (CON-1407). Expanded integration with Stock persistence support.
11. **Fuzz Testing**: Resolved (CON-1332 / GitHub #147). A weekly/manual cargo-fuzz regression workflow covers intent parsing, MuSig2 aggregation, anchoring receipt deserialization, and proof-request deserialization plus structural validation; when an optional proof envelope is present, its fail-closed contract and policy validation also runs. The proof-request target does not claim cryptographic BitVM2 proof verification; see [docs/FUZZING.md](FUZZING.md).
12. **SDK Integration**: Resolved (CON-1420). Added conxius-enclave-sdk as optional dependency.

## Open GitHub Issues (Cross-Repository)

### conxian-gateway (11 open)
| Priority | Issue | Action |
|----------|-------|--------|
| P0 | Publish TypeScript SDK to npm | Track |
| P1 | RGB Full stash resolver integration | Track |
| P1 | DLC CET construction path | Track |
| P1 | BitVM Groth16 verifier boundary | Track |
| Research | Babylon Cosmos SDK light client | Track |
| Research | DLC oracle integration (rust-dlc) | Track |
| Research | Liquid peg-in/peg-out E2E | Track |

### conxius-wallet (3 open)
| Priority | Issue | Action |
|----------|-------|--------|
| P1 | Technical debt reduction | Track |
| P1 | Strict CI/CD baseline | Track |
| Feature | Native Silent Payment (BIP-352) | Track |

## Current Focus: SDK Alignment

1. ✅ Add conxius-enclave-sdk dependency
2. ⚠️ Keep the optional SDK integration thin; migrate consumers directly to `conxius-enclave-sdk`
3. ✅ Removed duplicated Vault implementations from core; production MuSig2 and BitVM2 ownership is in `conxius-enclave-sdk`
4. ⚠️ Add feature flags for SDK integration
5. 📋 Update conxian-gateway to use SDK properly

## Recommendations

1. **Short-term**: Keep the optional SDK integration thin and guide consumers to migrate directly to `conxius-enclave-sdk`
2. **Medium-term**: Maintain lib-conxian-core as the home for unique protocol primitives while keeping production Vault functionality in `conxius-enclave-sdk`
3. **Long-term**: Consider merging lib-conxian-core into conxius-enclave-sdk or keeping as thin wrapper
