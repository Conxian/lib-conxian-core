# lib-conxian-core API Reference v0.2.12

## 1. Overview
This library provides the Rust-native API for Conxian protocol primitives. It is intended to be used as a dependency (`lib-conxian-core`) by the standalone Gateway, Wallet, and third-party integrators.

## 2. Core Modules

### Universal Chain Signing (`signing`)
The platform-neutral contract for SDK and Gateway signer adapters.
- `UniversalChainSigner`: capability-gated signing, address derivation, and complete signature verification.
- `SignRequest`, `AddressDerivationRequest`, and `VerificationRequest`: explicit target, algorithm, payload/digest, and derivation metadata.
- `SignerCapabilities`: versioned supported chain, algorithm, operation, and address-format declarations.
- `SigningError`: structured fail-closed and secret-safe error taxonomy.

See [docs/SIGNING_ARCHITECTURE.md](SIGNING_ARCHITECTURE.md) for ownership boundaries and chain examples. Core defines the contract only; SDK, Gateway, Wallet, and Nexus retain their documented runtime responsibilities.

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
- `Bip110Compliance`: Legacy aggregate size validator. `Bip110Compliance::new()` creates an
  enabled validator with canonical limits, while `Bip110Compliance::default()` is intentionally
  disabled (equivalent to `Bip110Compliance::disabled()`); callers must choose `new()` when they
  want enforcement.
- `Bip110Compliance::validate_transaction(...)`: The legacy API treats `Some(size)` as one full
  serialized OP_RETURN output ScriptPubKey and `None` as no OP_RETURN occurrence, so `None` skips
  OP_RETURN validation. Its `script_pubkey_size` argument is one full serialized non-OP_RETURN
  ScriptPubKey. Both measurements include the complete serialized script, including opcodes and
  data-push prefixes, rather than only the output payload.
- `Bip110Limits`: Canonical BIP-110 size-policy limits for core validation.
- `Bip110TransactionShape`: Serializable transaction-wide size metadata for the BIP-110 contract.
- `Bip110PreflightRequest` / `Bip110PreflightResult`: Versioned, serializable pre-construction and
  post-serialization request/result envelopes with fixed-width `u64` byte measurements and an
  explicit `Bip110MeasurementSource` provenance label. Pre-construction is caller-classified
  planning metadata, not full serialized transaction validation.
- `Bip110PreflightValidator`: Enabled, fail-closed composition layer over `Bip110Compliance`.
- `Bip110PreflightError` / `Bip110PreflightViolation`: Stable structural and indexed size findings
  with machine-readable codes, including phase mismatch and missing-measurement errors, field
  names, occurrence indices, actual bytes, and maximum bytes.
- `Bip110PreflightMeasurements::taproot_control_block_sizes_bytes`: Separate fixed-width control-
  block sizes; `257` is admissible and `258` returns a control-block-specific violation. Size
  admissibility does not validate BIP-341 shape, commitments, or cryptography.
- `Bip110OperationContext`: Stable context strings; only `bitcoin_transaction` is supported in
  API version 1, while known protocol-specific and unknown contexts fail closed.
- See [`BIP110_ALIGNMENT.md`](BIP110_ALIGNMENT.md) for the full rule matrix, proposal/deployment assumptions, byte-measurement semantics, context exceptions, and downstream ownership.
- `PartnerLead`: Intake model for ecosystem integrations (CON-63).
- `Chain`: Supported networks including Bitcoin, Stacks, CosmosHub, Solana, and Eclipse (ADR-006).

Preflight consumers must provide classified measurements before treating a result as usable. Empty
vectors are valid only for an explicitly present generic Bitcoin transaction with zero constrained
occurrences; omitted measurement data, phase/source mismatches, unsupported contexts, and unknown
contexts never produce compliant success. Core owns these serializable contracts and deterministic
findings. Transaction parsing/classification remains with SDK, Wallet, or another transaction-aware
adapter; Gateway owns orchestration, persistence, routing, and external side effects. This
repository does not claim downstream integration is complete.

Example of an explicitly classified, pre-construction request:

```rust
use lib_conxian_core::control_model::{
    Bip110OperationContext, Bip110PreflightMeasurements, Bip110PreflightPhase,
    Bip110PreflightRequest,
};

let measurements = Bip110PreflightMeasurements::new_with_control_block_sizes(
    vec![32], vec![], vec![34], vec![64], vec![257],
);
let request = Bip110PreflightRequest::new(
    Bip110PreflightPhase::PreConstruction,
    Bip110OperationContext::BitcoinTransaction,
    measurements,
);
let result = request.validate();
assert!(result.is_compliant);
```

Use `Bip110PreflightRequest::without_measurements` when classified data is unavailable; it returns
the stable `missing_measurement_data` finding rather than treating empty vectors as evidence.

### Protocol Verification (`verifier`)
Platform-neutral contracts for downstream chain verifiers.
- `ProtocolVerifier<B>`: Enforceable façade for state-proof verification, latest verified block references, and transaction finality.
- `ProtocolVerifierBackend`: Lower-level backend hooks; consumers must call the façade rather than the hooks directly.
- `DynProtocolVerifier`: Ergonomic `Box<dyn ProtocolVerifierBackend>` façade alias for runtime-selected backends.
- `VerifierCapabilities`: Explicit supported chains, proof formats, verification classes, finality classes, and trust tiers.
- Every façade success path requires returned trust, verification, and finality metadata to be advertised by the stored capabilities and binds result provenance to `VerifierCapabilities.verifier_id`.
- `ChainId::from_chain` and `control_model::chain_family_for`: Canonical chain-family mapping with checked explicit construction and deserialization validation.
- `validate_proof_verification_result_at`: Request-aware chain, block, proof-format, and exact state-root postconditions.
- `compute_evidence_binding_hash`: Versioned, domain-separated, manually length-prefixed structural binding for request/proof/envelope evidence.
- `ProtocolVerifierError`: Fail-closed taxonomy for invalid families, unsupported, malformed, stale, unavailable, expired, future-dated, unbound, unadvertised result policy, verifier-identity, non-final, and policy-blocked evidence.
- `LatestVerifiedBlock` and `TransactionFinalityResult`: Provenance-bearing results with invariant validation.

Runtime proof acquisition, chain observation, light clients, persistence, and orchestration remain in Nexus, Gateway, or downstream adapters. The structural binding is not authenticity; downstream signatures, attestations, light clients, or verifier-set proofs remain required. See [docs/architecture/PROTOCOL_VERIFIER.md](architecture/PROTOCOL_VERIFIER.md).

## 5. Deterministic Core-to-Downstream Fixtures (CON-1505)

This is the initial Core-owned serialized-contract checkpoint. The first
repository-local integration layer is owned by Core and lives under
`tests/fixtures/` with its harness in `tests/core_to_downstream_integration.rs`:

- `signing_boundary.json` covers the versioned signer capability advertisement,
  `SignRequest`/`SignResponse` shapes, and unsupported chain, algorithm, and
  operation errors.
- `verifier_boundary.json` covers proof and finality request/result shapes,
  capability advertisements, malformed proof input, stale evidence returned by
  a named test-only backend, and policy rejection of degraded evidence.
- `bip110_preflight.json` covers compliant and non-compliant measurements plus
  rejection of an unsupported API version.
- `adapter_contracts.json` covers representative `TxParams`, chain-family, trust,
  address, and fee contract metadata for local adapter doubles/implementations.

Fixtures are synthetic, deterministic, and safe to commit. They contain no
credentials, private key material, production principals, RPC data, network
calls, environment requirements, hardware behavior, or downstream checkouts.
The verifier and adapter cases assert structural Core contracts only; they do
not claim authoritative cryptographic verification. The stale-evidence case
preserves the typed `ProtocolVerifierError::StaleReference` shape through a
test-only backend because the current Core façade does not acquire live evidence.

The compatibility assumptions recorded by this checkpoint are limited to the
Core package `lib-conxian-core` `0.2.12`, Rust `1.85`, the default feature set
(`default = []`),
`UNIVERSAL_CHAIN_SIGNER_API_VERSION = 1`,
`BIP110_PREFLIGHT_API_VERSION = 1`, and
`PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION = 1` with the Core evidence-binding
domain constant. The known optional production SDK assumption is the existing
`conxius-enclave-sdk` `2.0.11` declaration; this fixture layer deliberately does
not compile or force that optional SDK path. This checkpoint does not claim
direct compile compatibility or revision pins for `conxius-enclave-sdk`,
`conxian-gateway`, or `conxian-nexus`. Per CON-1505, pinned downstream fan-out
is intentionally deferred until the UCS and ProtocolVerifier APIs stabilize;
no Gateway or Nexus pin is asserted here.

Run the focused layer locally with:

```text
cargo fmt --all -- --check
cargo test --locked --test core_to_downstream_integration
cargo test --locked --test universal_chain_signer
cargo test --locked --test protocol_verifier
cargo test --locked --test bip110_preflight
```

The existing workspace CI remains the broader validation path. Live downstream
CI fan-out is intentionally deferred, opt-in, and expected to use pinned
consumer revisions until these Core contracts stabilize.

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
