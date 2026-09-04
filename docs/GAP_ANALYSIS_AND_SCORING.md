# Gap Analysis & Implementation Scoring (CON-1305)

> **Session Note**: Updated 2026-07-15 to reflect SDK integration and crate relationship mapping.
>
> **See Also**: [ALIGNMENT.md](./ALIGNMENT.md) for comprehensive ecosystem alignment.

This document maps identified protocol gaps to research status and implementation priority scoring.

## Current Core boundary (2026-08-01)

The SDK and proposal scores below must not be read as claims that
`lib-conxian-core` itself provides production verification. In Core:

- FROST share generation, distribution, and aggregation are typed
  `Unsupported` boundaries until an audited implementation is supplied.
- Enclave DER handling parses container shape only; it is not certificate-chain
  or hardware-attestation verification. Production attestation belongs in
  `conxius-enclave-sdk`.
- BIP-322 handles address/base64/witness shape only; it does not perform
  cryptographic script or signature verification.
- Fedimint point reconstruction is a deterministic primitive, not
  provider-backed mint/note/status verification. Authenticated mint status is
  unavailable without a provider.

## Critical Discovery: SDK Integration (historical v0.2.10 baseline)

**The production Vault SDK is in [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) v2.0.17**, NOT in this repository.

### Crate Relationship Matrix

| Crate | Version | Purpose | Status |
|-------|---------|---------|--------|
| `conxius-enclave-sdk` | 2.0.17 | **Production Vault SDK** - Hardware signing, attestation, FROST DKG, Ark, BitVM2 | ✅ Production |
| `lib-conxian-core` | 0.3.3 | **Protocol primitives** - Types, invariants, chain adapters | ⚠️ Fail-closed boundary |
| `conxian-gateway` | 0.1.4 | Runtime orchestration and middleware | ⚠️ WIP |

### Local Implementations vs SDK

| Local Module | SDK Module | Gap Status |
|--------------|------------|-------------|
| Removed in-core MuSig2/BitVM2/Vault implementations | `conxius-enclave-sdk` | ✅ **SDK-owned** - production signing, sessions, attestation, and BitVM2 verification live in the SDK |
| `fuzz/fuzz_targets/musig2_aggregate.rs` | upstream `musig2::KeyAggContext` | ✅ **Dependency-level fuzz coverage** |
| `fuzz/fuzz_targets/proof_request_validate.rs` | `src/verifier.rs::ProofVerificationRequest::validate` | ✅ **Structural and contract/policy validation; no cryptographic BitVM2 proof verification** |
| `src/protocol/frost.rs` | `conxius-enclave-sdk` FROST implementation | ⚠️ Core rejects placeholders; production share/distribution/aggregation is SDK-owned |
| `src/control_model/` | N/A | ✅ Unique to lib-conxian-core |

## SDK Capabilities (not implementations in Core)

### SDK Modules (via conxius-enclave-sdk)

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
| **FROST Threshold (G-14)** | 40 | 25 | 30 | **95** | **SDK-owned; core rejects placeholders** |
| **DLC Primitives (G-06)** | 35 | 25 | 30 | **90** | **Implemented** (Equation & CET structure validation; execution downstream) |
| **Hardware Attestation (G-17)**| 35 | 20 | 30 | **85** | **Implemented** (SDK; Core DER parse-only) |
| **Babylon Staking (G-43)** | 35 | 25 | 30 | **90** | **Implemented** |
| **BitVM2 Multi-Party (G-11)**| 40 | 30 | 20 | **90** | **Implemented** (SDK) |
| **BIP-322 (G-09)** | 40 | 30 | 20 | **90** | **Strict parser; verifier downstream** |
| **Fedimint (G-16)** | 30 | 25 | 25 | **80** | **Implemented** (SDK; Core provider status unavailable) |
| **Silent Payments (G-05)** | 35 | 25 | 20 | **80** | **Implemented** (SDK) |
| **RGB Integration (CXIP-20)** | 35 | 20 | 30 | **85** | **Fail-closed adapter boundary** |
| **Fuzz Testing (CON-1332 / GitHub #147)** | 30 | 30 | 20 | **80** | **Implemented** (4 bounded targets; weekly/manual CI) |
| **BitVMX (G-44)** | 40 | 15 | 30 | **85** | Researching |
| **BitVM3 (G-20)** | 40 | 10 | 30 | **80** | Directional |
| **ZKCP (G-50)** | 35 | 15 | 20 | **70** | Researching |

## Gap Identification & Resolution
1. **Universal Chain Adapters**: Core adapters expose typed fail-closed boundaries for Cosmos, Solana, Move, and Substrate; verified light-client backends remain downstream (CXIP-21).
2. **BitVM2 Multi-Party**: Resolved (CON-1306). Production MuSig2-based Taproot tree aggregation and BitVM2 verification are owned by `conxius-enclave-sdk`; this crate no longer carries the Vault implementation.
3. **BIP-322**: Fail-closed in core (CON-1509). It strictly parses supported input shapes but advertises no script verifier until an audited implementation is available.
4. **FROST Round 2**: Fail-closed in core (CON-1509). Production DKG and signing remain SDK-owned; placeholder operations cannot succeed.
5. **Hardware Attestation**: Core parses DER containers only; certificate-chain and hardware-attestation verification is SDK-owned.
6. **MuSig2 Signature Aggregation**: Resolved (G-10). Production signing and session aggregation are owned by `conxius-enclave-sdk`; this crate retains only protocol primitives and direct dependency-level fuzz coverage.
7. **Fedimint**: Core provides deterministic point reconstruction only; authenticated mint, note, and status verification requires a provider, and mint status is unavailable without one.
8. **Silent Payments**: Resolved (G-05). Hardened scanning logic with real ECC point math.
9. **DLC**: Equation verification is retained, intent-bound policy checks are typed, and CET structure/payout validation is hardened (`DlcManager::validate_cet_structure`); execution, funding, and finality remain downstream (CON-1509).
10. **RGB**: Fail-closed adapter boundary (CON-1509, CON-1407); Stock/node-backed verification remains follow-up work, contract lookups reject empty/whitespace inputs with `InvalidContractId`, and Shadow mode is non-authoritative.
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

## Infrastructure & Persistence Gap Mapping (2026-08-19 Update)

### Cloud DB Alignment (`org-silent-sun-00457600`)
- **`corelibs` (`sparkling-sunset-69236559`, us-east-2)**: Core protocol state roots & invariant verification schemas.
- **`Software dev kit` (`weathered-night-98492579`, us-east-2)**: Vault SDK hardware attestation logs & FROST DKG session tracking.
- **`Business Operating System` (`noisy-flower-17484435`, us-east-2)**: BOS policy rules, risk profiles, and audit event logs.
- **`market` (`small-math-44741750`, eu-central-1)**: Intent matching, liquidity routing, and ERC-7683 solver registry.
- **`Gateway` (`noisy-cloud-41146057`, ap-southeast-1)**: Gateway API session tracking and rate-limit counters.
- **`Conxian Nexus` (`orange-paper-76209725`, eu-central-1)**: zkVM proof verification state & state root commitments.

### Resolution & Next Step Priorities
1. **DLC Verification Edge Cases**: Expand `src/protocol/dlc.rs` unit test coverage for invalid collateral, zero expiry block, invalid nonces, and corrupted signatures.
2. **SDK Migration**: Maintain `lib-conxian-core` as a Zero Secret Egress protocol library while recommending `conxius-enclave-sdk` v2.0.17 for enclave hardware signing.
3. **Fuzz & Verification Coverage**: Maintain 100% compliance across all 4 fuzz targets and verification scripts.


## Session 62 Research & Infrastructure Synthesis (2026-09-03)

### Current Protocol Baseline (v0.3.3)
- **Core Verification Suite**: 128 core protocol unit and doc tests passing (100% success rate across core library).
- **Python Verification Guard Suite**: 69 test cases passing in `scripts/tests/` (100% pass rate).
- **Zero Architectural Contamination**: `scripts/verify_contamination_guard.py` and `scripts/verify_tracked_artifacts.py` confirm zero forbidden I/O leaks or unindexed tracking risks.

### Cloud DB & Render Service Fleet Topology
- **Neon Cloud Fleet (`org-silent-sun-00457600`)**:
  - `corelibs` (`sparkling-sunset-69236559`, us-east-2, PG18): Protocol state roots & invariant verification schemas.
  - `Software dev kit` (`weathered-night-98492579`, us-east-2, PG18): Vault SDK session state, DKG logs, and attestation proofs.
  - `Business Operating System` (`noisy-flower-17484435`, us-east-2, PG18): Enterprise risk control plane, policy enforcement & billing.
  - `market` (`small-math-44741750`, eu-central-1, PG18): Cross-chain orderbooks, ERC-7683 solver registry, and liquidity routing.
  - `Gateway` (`noisy-cloud-41146057`, ap-southeast-1, PG18): Gateway API runtime state, rate limiting, and client sessions.
  - `Conxian Nexus` (`orange-paper-76209725`, eu-central-1, PG17): zkVM proof aggregation, state roots, and logical replication.
- **Render Team Workspace (`tea-d6u0edngi27c73dvhsg0`)**:
  - `conxian-business-static-docs` (`srv-d9h2nu2b6mfs738i6gb0`): Static documentation gateway.
  - `conxian-business` (`srv-d9gam3m1a83c73bmrfc0`): Enterprise backend service.
  - `conxian-ui-prod` (`srv-d96fl2mq1p3s73c2e8k0`): Production user dashboard.
  - `conxian-labs-static-v1` (`srv-d8fmr7v40ujc73b7ba8g`): Corporate site landing.
  - `conxian-ui-hco6` (`srv-d7b0el3uibrs73b2qjg0`): Staging interface deployment.

## Session 66 Research & Protocol Invariant Hardening (2026-09-04)

### Current Protocol Baseline (v0.3.3)
- **Rust Workspace Verification Suite**: 262 total Rust workspace tests passing (100% pass rate).
- **Python Verification Guard Suite**: 69 test cases passing in `scripts/tests/` (100% pass rate).
- **Covenant Invariant Hardening**: Enhanced `CovenantManager` in `src/protocol/covenant.rs` with `generate_cat_vault_script_checked` returning typed `CovenantError` variants on invalid pubkey lengths or malformed target hash inputs.
- **Zero Architectural Contamination**: `scripts/verify_contamination_guard.py` and `scripts/verify_tracked_artifacts.py` confirm zero forbidden I/O leaks or unindexed tracking risks.
