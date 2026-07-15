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
| `lib-conxian-core` | 0.2.10 | **Protocol primitives** - Types, invariants, chain adapters | ⚠️ Prototype |
| `conxian-gateway` | 0.1.4 | Runtime orchestration and middleware | ⚠️ WIP |

### Local Implementations vs SDK

| Local Module | SDK Module | Gap Status |
|--------------|------------|-------------|
| `src/musig2.rs` | `src/protocol/musig2.rs` | ⚠️ **Simplified** - SDK uses real musig2 crate with BIP-327 |
| `src/bitvm2.rs` | `src/protocol/bitvm2.rs` | ⚠️ **Stub** - SDK has full challenge orchestration |
| `src/wallet.rs` | `src/enclave/` | ⚠️ **Basic** - SDK has hardware attestation |
| `src/sdk_primitive.rs` | N/A | ⚠️ **Deprecated** - Use conxius-enclave-sdk directly |
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
| **Fuzz Testing (CON-147)** | 30 | 30 | 20 | **80** | **Implemented** |
| **BitVMX (G-44)** | 40 | 15 | 30 | **85** | Researching |
| **BitVM3 (G-20)** | 40 | 10 | 30 | **80** | Directional |
| **ZKCP (G-50)** | 35 | 15 | 20 | **70** | Researching |

## Gap Identification & Resolution
1. **Universal Chain Adapters**: Skeletal implementation complete for Cosmos, Solana, Move, and Substrate (CXIP-21).
2. **BitVM2 Multi-Party**: Resolved (CON-1306). Implemented MuSig2-based Taproot tree aggregation.
3. **BIP-322**: Resolved (CON-1266). Hardened universal message signing logic.
4. **FROST Round 2**: Resolved (CON-1329). Moving from skeletal generation to encrypted share distribution.
5. **Hardware Attestation**: Resolved (CON-1329). Implementing X.509 DER parsing for enclave certificate chains.
6. **MuSig2 Signature Aggregation**: Resolved (G-10). Transitioning from dummy aggregation to real sum-of-scalars logic.
7. **Fedimint**: Resolved (G-16). Transitioning to real cryptographic blinding via `fedimint-client-wasm`.
8. **Silent Payments**: Resolved (G-05). Hardened scanning logic with real ECC point math.
9. **DLC**: Resolved (G-06). Hardened oracle attestation verification.
10. **RGB**: Resolved (CON-1407). Expanded integration with Stock persistence support.
11. **Fuzz Testing**: Resolved (CON-147). Established fuzzing infrastructure for intent and MuSig2.
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
2. ✅ Deprecate local VaultSDK
3. ⚠️ Consider removing duplicated implementations (musig2, bitvm2)
4. ⚠️ Add feature flags for SDK integration
5. 📋 Update conxian-gateway to use SDK properly

## Recommendations

1. **Short-term**: Mark local VaultSDK as deprecated, guide users to conxius-enclave-sdk
2. **Medium-term**: Remove duplicated code from lib-conxian-core, keep only unique protocol primitives
3. **Long-term**: Consider merging lib-conxian-core into conxius-enclave-sdk or keeping as thin wrapper
