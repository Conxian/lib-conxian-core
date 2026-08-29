# Conxian Agent Guidelines: lib-conxian-core (v0.3.2 — Session 58, Aug 2026)

This repository is the canonical home of shared protocol primitives and the Core type system for the Conxian ecosystem. It is a "protocol-first" library.

> **Note:** The production Vault SDK (hardware-backed signing, attestation, FROST DKG, BitVM2) lives in [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) (v2.0.16), NOT in this repository. See [docs/MIGRATION.md](docs/MIGRATION.md).

> **Session 52 review (2026-08-05):** Audited AGENTS.md, README.md, and PHASE1_ISSUES_ROADMAP.md. Fixed stale VaultSDK references, SDK version pins (v2.0.11→v2.0.14), SDK re-export module counts (50→70 with signing category), and Phase 1 status (6/9 CORE issues now closed). Blocked modules documented: Rails (6, pub(crate)), frost_crypto, wasm_bindings, android_strongbox, cloud.

> **Session 52 AWS Nitro POC (2026-08-05):** Cross-referenced both repos (core v0.3.1 + SDK v2.0.16). Built `enclave-poc/` — a complete Nitro Enclave signing demo that exercises real Core types, the adapter boundary, and SDK's `EnclaveManager` trait. All 227 tests pass. POC demonstrates: Strict-tier Bitcoin signing with BIP-110 preflight ✅, ObserverOnly rejection ✅, 5-chain signing flow ✅. AWS Nitro deployment guide in `enclave-poc/README.md`. Docker artifacts in `enclave-poc/docker/`.

> **Session 52 Nitro CI + Extended POC (2026-08-05):** Created `.github/workflows/nitro-enclave-ci.yml` with build-test-docker-provision pipeline. Extended POC from 3→6 scenarios (error injection, key rotation, replay detection). Saved AWS secrets to GitHub repo secrets. Documented AWS permissions matrix — `botshelo` IAM user can read EC2 + manage SGs/roles but cannot launch instances or create OIDC providers. Full Nitro deployment requires either (a) adding `ec2:RunInstances`,`ec2:TerminateInstances`,`ec2:CreateKeyPair`,`iam:CreateInstanceProfile`,`iam:AddRoleToInstanceProfile`,`iam:PassRole` to the user, or (b) creating a `github-actions-nitro-provisioner` IAM role with OIDC trust for GitHub Actions. See [AWS Permissions Matrix](#aws-nitro-permissions-matrix) below.

> **Session 52 end-to-end Nitro provisioning (2026-08-05):** Pipeline now fully operational: build-test → docker-build → provision-nitro → teardown. 7 spot instance types tried (fallback chain), c5.xlarge used. SG `conxian-nitro-sg`, instance profile `conxian-nitro-enclave-profile`, role `conxian-nitro-enclave-role` all created. Dynamic AMI/subnet discovery. IAM policy attached to `botshelo`. Main CI + SDK compat workflows use rustc 1.97.1.

## Architectural Boundaries (CON-700)
- **Core (`src/`):** Ownership of canonical types, state machines, invariant validation, and interface contracts.
- **Gateway:** Runtime orchestration, persistence, and external side effects live in the standalone `conxian-gateway` repository.
- **Rule:** If a change needs network IO, database access, or environment-specific branching, it belongs in the Gateway, not here. This is automatically enforced by `scripts/verify_contamination_guard.py`.

## Trust Policy Enforcement (CON-791)
Ensure all cross-domain bridge or messaging metadata aligns with the approved trust-tier taxonomy in `control_model.rs`:
- `Strict` (T1)
- `Managed` (T2)
- `Expedient` (T3)

## Protocol Coverage — SDK → Core Alignment

The Conxius Enclave SDK (`conxius-enclave-sdk` v2.0.16) defines the canonical 42-chain `AssetRegistry` and 52 protocol modules (24 blockchain + 28 infrastructure). lib-conxian-core is the **shared type system** consumed by both nexus and gateway — it must provide canonical types for every chain and protocol the ecosystem touches.

### Core Type Coverage Requirements

| Domain | Core Types | Consumers |
|--------|-----------|-----------|
| Bitcoin L1 | Block, Tx, Utxo, Script | nexus, gateway |
| Stacks | StacksBlock, ClarityValue | nexus, gateway |
| EVM-compatible | EvmBlock, EvmTx, EvmReceipt | nexus, gateway |
| Cosmos | CosmosBlock, CosmosTx | nexus |
| Lightning | LightningInvoice, LnPayment, LnChannel | nexus, gateway |
| MMR | MmrNode, MmrProof, StateRoot | nexus |
| RGB | RgbContract, RgbConsignment, RgbStash | gateway, nexus |
| BitVM2 | BitVmProof, BitVmGate, BitVmChallenge | gateway, nexus |
| DLC | DlcContract, DlcOracle, DlcOutcome | gateway, nexus |
| Fedimint | FedimintModule, FedimintConsensus | gateway, nexus |
| Canton | UniversalContractRef, CantonDomainRef, CantonStateTranslation | gateway |
| Machine Economy | MachineIdentity, MachineRwaRevenue, M2MSettlement | gateway |
| Settlement | SettlementRail, RailPlan, SettlementReceipt | gateway, platform |
| Identity | DidDocument, DidResolution, IdentityProof | gateway, platform |

### Trust-Tier to Protocol Mapping

| Tier | Protocols | Characteristics |
|------|-----------|----------------|
| T1 Strict | Bitcoin L1 finality, BitVM2 SNARK verification, RGB state proofs | Full validation, no trust assumptions |
| T2 Managed | Lightning channel state, Fedimint consensus, Canton Daml state | Consortium/multi-sig governance |
| T3 Expedient | EVM L2 bridges (Arbitrum/Base/OP), Cosmos IBC, Solana | Optimistic or fast-finality, economic security |

## Module Catalog (Session 48 — Aug 2026)

### Core Modules

| Module | Path | Key Public Types | Status |
|--------|------|-----------------|--------|
| adapters | `src/adapters/mod.rs` | StateProofError, chain adapter abstraction | ✅ |
| anchoring | `src/anchoring.rs` | AnchoringPublisher, TablelandAnchoringPublisher, OnChainAnchoringPublisher | ✅ |
| babylon | `src/babylon/mod.rs` | BabylonAdapter, StakingIntent | ✅ |
| cjcs | `src/cjcs.rs` | Canonical Job Card System types | ✅ |
| contract_bridge | `src/contract_bridge.rs` | ClarityCall, ContractBridge, SignedContractCall | ✅ |
| control_model | `src/control_model/` | BIP110, TrustTier, LifecycleState, RiskProfile, Chain, ChainFamily | ✅ |
| deployment | `src/deployment.rs` | DeploymentPlan, contract deployment configuration | ✅ |
| fedimint | `src/fedimint/mod.rs` | FedimintAdapter, FedimintMint | ✅ |
| protocol | `src/protocol/` | covenant, dlc, frost, intent | ✅ |
| verifier | `src/verifier.rs` | ProtocolVerifier, ProofVerificationRequest, CapabilityVerifier, ChainId, BlockReference, VerifiedBlockReference, TransactionFinalityStatus | ✅ |

### CXIP 20 Extended Modules

| Module | Path | Key Public Types | Status |
|--------|------|-----------------|--------|
| bitcoin | `src/bitcoin/mod.rs` | bip322, taproot, liquid_adapter, SilentPaymentScanner | ✅ |
| crypto | `src/crypto/mod.rs` | CryptoStubError, advanced cryptography utilities | ✅ |
| enclave | `src/enclave/mod.rs` | AttestationCertificate, EnclaveVerificationError | ✅ |
| lightning | `src/lightning/mod.rs` | LightningAdapter, LightningPaymentIntent/Event/State, LightningNode | ✅ |
| rgb | `src/rgb/mod.rs` | RGBAdapter, RGBStockAdapter, RGBSkeletonAdapter, RGBRuntime | ✅ |
| signing | `src/signing.rs` | SigningAlgorithm, SigningTarget, SignerCapabilities | ✅ |
| stacks | `src/stacks/mod.rs` | sBTC, StacksNakamoto, StacksAdapter, SBTCBridge | ✅ |

### Re-exported at Crate Root

- `ClarityCall`, `ContractBridge`, `SignedContractCall` (from `contract_bridge`)
- 30+ verifier types (from `verifier`)

### SDK Re-exports (Session 57 — Full Alignment with conxius-enclave-sdk v2.0.16)

The `sdk` module (`src/sdk.rs`) re-exports ALL 70 accessible conxius-enclave-sdk modules organized by category.
Enable via Cargo features:

```toml
lib-conxian-core = { version = "0.3.1", features = ["full-sdk"] }
```

| Category | Feature Flag | Modules | SDK Re-export Path |
|----------|:-----------:|:-------:|-------------------|
| Blockchain | `sdk-blockchain` | 24 | `sdk::blockchain::{ark, asset, babylon, bip110, bip322, bitcoin, bitvm, bitvm2, cctp, covenant, credit, dlc, ethereum, fiat, frost, lightning, mmr, musig2, rgb, sidl, solana, stacks, statechain, swap_router}` |
| Cross-cutting | `sdk-cross-cutting` | 15 | `sdk::cross_cutting::{a2p, account_abstraction, business, chain_abstraction, control_model_adapter, economy, identity, intent, job_card, opportunity, settlement, settlement_service, solver, stablecoin_orchestrator, zkml}` |
| Nexus | `sdk-nexus` | 2 | `sdk::nexus::{fedimint, roast}` |
| Infrastructure | `sdk-infrastructure` | 5 | `sdk::infrastructure::{config, serde_big_array, state, telemetry, wasm_support}` |
| Signing | `sdk-signing` | 13 | `sdk::signing::{bip110, bip322, bitvm2, covenant, dlc, lightning, musig2, statechain, taproot, threshold, ucs, wasm_runtime, zkml}` |
| Enclave | `enclave` | 11 | `sdk::enclave_sdk::{android_authorization, attestation, durable_replay, nitro, proof, proofs, replay_guard, trust, trust_contracts}` + crate-root `EnclaveManager, SignRequest, SignResponse, SigningAlgorithm, ConclaveError, ConclaveResult` |

**Blocked modules:**
- Rails (6): `pub(crate)` in SDK — cannot re-export (bisq, boltz, changelly, wormhole, ntt, x402)
- `frost_crypto`: `#[cfg(feature = "frost-crypto")]` in SDK
- `wasm_bindings`: `#[cfg(target_arch = "wasm32")]` in SDK
- `android_strongbox`, `cloud`: `#[cfg(any(test, feature = "development-simulators"))]` in SDK

**Meta-feature:** `full-sdk` enables all 6 categories at once. The full `conxius_enclave_sdk` crate is also
re-exported at `sdk::conxius_enclave_sdk` for direct access.

## Consumer Wiring

| Consumer | Modules Used | Wiring Path |
|----------|-------------|-------------|
| conxian-nexus | 12/17 | `compat::core_bridge::core_types` re-exports: control_model, signing, verifier, anchoring, bitcoin(taproot,bip322), protocol(dlc,frost,covenant,intent), lightning, adapters |
| conxian-gateway | Own `conxian_core` + contract_bridge | Separate operational types crate; uses contract_bridge types through engine |
| conxius-wallet | None directly | Uses `conxius-enclave-sdk` for signing (feature-gated via silent-payments crate) |
| conxius-platform | None directly | TS orchestration; CI scripts reference canonical paths |
| conxius-orbit | contract_bridge (planned) | CLI for deploying Stacks contracts; contract_bridge will provide typed principals |

### Underutilized Modules (No Direct Consumer)

| Module | Consumer Gap | Recommendation |
|--------|-------------|----------------|
| `crypto` | No consumer uses core crypto stubs | Gateway should use for transaction hashing in mempool orchestrator |
| `enclave` | No consumer uses attestation types | Nexus executor should verify attestation before processing proofs |
| `deployment` | Only used in internal tests | Orbit CLI should consume DeploymentPlan for contract deploy orchestration |
| `cjcs` | No consumer uses Job Card types | Platform should use for SLA enforcement and gap job card generation |
| `babylon` | Nexus uses trust tier but not staking | Gateway treasury monitor should integrate Babylon staking lifecycle |
| `fedimint` | No consumer uses mint consensus types | Gateway should validate Fedimint state proofs against trust tiers |

## Workflow Instructions
- **Verification:** Always run `cargo test --workspace` to verify changes.
- **ZSE:** Adhere to Zero Secret Egress standards. Never track environment files or private keys.
- **Source of Truth:** Refer to `bitcoinlayers.org` for the latest Bitcoin Layer 2 research.
- **Protocol Coverage:** When adding a new chain or protocol to the ecosystem, first add canonical types here, then implement adapters in gateway/nexus.


## Key References
- **Release process**: `docs/RELEASE_PROCESS.md`
- **Compatibility matrix**: `docs/COMPATIBILITY.md`
- **Architecture boundaries**: `docs/ARCHITECTURE_BOUNDARIES.md`
- **Session archive**: `docs/archive/AGENTS_archive_session_58.md` (AWS Nitro matrix, session history)
- **Build**: `cargo build --locked && cargo test --locked && cargo clippy -- -D warnings`
