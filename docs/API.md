# lib-conxian-core / Vault SDK API Reference v0.2.10

## 1. Overview
This library provides the Rust-native API for Conxian protocol primitives. It is intended to be used as a dependency (`lib-conxian-core`) by the standalone Gateway, Wallet, and third-party integrators.

## 2. Core Modules

### Vault SDK (`sdk_primitive`)
The primary interface for hardware-anchored signing and policy enforcement.
- `VaultSDK::new(wallet: Wallet, policy: SigningPolicy)`: Initializes a new SDK instance.
- `VaultSDK::sign_with_policy(tx_id: &str, amount_sats: u64, destination: &str)`: Validates and signs a transaction after policy verification.

### Deployment & Artifacts (`deployment`)
Shared schemas for machine-readable execution records (CON-1237).
- `DeploymentManifest`: Canonical record of a completed or failed deployment.
- `VerificationResult`: The result of a post-deployment verification check with evidence.

### Lightning (`lightning`)
Resilience and recovery models for Lightning operations.
- `LightningPaymentState`: Canonical payment lifecycle states (SRL-1).
- `LightningFailureClass`: Taxonomy for failure handling and retries (SRL-7).
- `LightningMetrics`: Observability data for node health and liquidity (SRL-9).
- `LightningAdapter`: Core interface for production-grade backends (SRL-10).

### Control Model (`control_model`)
Canonical types for protocol orchestration and trust.
- `StateProposal`: Unified model for external settlement triggers (CON-162).
- `TrustTier`: Approved classification for bridge/messaging security (CON-791).
- `Bip110Limits`: Canonical BIP-110 byte-size limits for core validation.
- `Bip110TransactionShape`: Serializable transaction-size metadata validated by the core contract.
- `PartnerLead`: Intake model for ecosystem integrations (CON-63).
- `Chain`: Supported networks including Bitcoin, Stacks, CosmosHub, Solana, and Eclipse (ADR-006).

### Anchoring (`anchoring`)
Models for decentralized state persistence.
- `AnchoringRequest`: Payload for committing state roots to Tableland or L1.
- `AnchoringReceipt`: Cryptographic proof of state commitment.

### Bitcoin (`bitcoin`)
Advanced Bitcoin-native primitives.
- `MuSig2`: BIP327-compliant key aggregation, signature aggregation, and signing (CON-145, CON-1270).
- `BitVM2`: Segment generation and optimistic fraud-proof verification (CON-464).
- `BIP-322`: Universal message signing and verification (G-09).

## 3. Trust Tier Policy (CON-791)
The library enforces explicit trust-tier metadata for all cross-domain operations:
- **T1: Strict**: Sovereign verified (e.g., IBC light-clients).
- **T2: Managed**: Hybrid verified with independent attesters.
- **T3: Expedient**: Attester network with caps and kill-switches.
- **T4: ObserverOnly**: Not allowed in production.

## 4. Integration Guidelines
Implementation details for runtime orchestration, network IO, and database persistence live in the standalone `conxian-gateway` repository. This library focuses exclusively on stable interfaces and protocol-bearing logic.

### Protocol Primitives (`protocol`)
Advanced protocol support for multi-party and cross-chain coordination.
- `IntentManager::rank_bids(bids: &[Bid])`: Ranks ERC-7683 intent solver bids.
- `FrostManager::generate_shares(threshold: u32, total: u32)`: Generates FROST key shares.
- `CovenantManager::generate_cat_vault_script(pubkey: &[u8], target_hash: &[u8])`: Generates OP_CAT recursive covenants.
- `DlcManager::create_intent(oracle_pubkey: &[u8], collateral: u64, outcome: [u8; 32], expiry: u32)`: Creates DLC intents for native Bitcoin finance (G-06).

### Universal Chain Adapters (`adapters`)
CXIP-21 interface for cross-chain orchestration.
- `UniversalChainAdapter`: Trait for uniform multi-chain support.
- `BitcoinAdapter`: Native UTXO support.
- `EvmAdapter`: Ethereum, Base, etc.
- `CosmosAdapter`: IBC-enabled networks.
- `SolanaAdapter`: SVM support.
- `BabylonAdapter`: Institutional Bitcoin staking (G-43).
