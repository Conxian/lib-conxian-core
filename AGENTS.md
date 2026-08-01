# Conxian Agent Guidelines: lib-conxian-core (v0.3.1 — Session 48, Aug 2026)

This repository is the canonical home of the **Vault SDK** and shared protocol primitives. It is a "protocol-first" library.

## Strategic Priority (Vault SDK)
Prioritize the development and hardening of the `VaultSDK` primitive (`src/sdk_primitive.rs`). This is the primary commercial interface for the Conxian ecosystem.

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

The Conxius Enclave SDK (`lib-conclave-sdk` v0.3.1) defines the canonical 42-chain `AssetRegistry` and 46 protocol modules. lib-conxian-core is the **shared type system** consumed by both nexus and gateway — it must provide canonical types for every chain and protocol the ecosystem touches.

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
- `EnclaveManager`, `SignRequest`, `SignResponse`, `SigningAlgorithm` (from `conxius-enclave-sdk`, `enclave` feature only)
- 30+ verifier types (from `verifier`)
- `ConclaveError`, `ConclaveResult` (from `conxius-enclave-sdk`, `enclave` feature only)

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
