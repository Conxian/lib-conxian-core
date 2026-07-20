# Migration Guide: lib-conxian-core v0.2.x → v0.3.0

> **Status**: As of v0.2.11, the deprecated Vault SDK modules have been removed from `lib-conxian-core`.
> All Vault SDK functionality is now available in the production [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk).

## Overview

Starting with v0.2.11, `lib-conxian-core` no longer includes Vault SDK functionality (hardware-backed signing, attestation, policy enforcement, MuSig2, BitVM2). This functionality has moved to the production [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate.

## ProtocolVerifier hardening (pre-publication)

The verifier contract introduced by #180/PR #185 is corrected before the next
publication. This change is intentionally documented as an unreleased API
break; crates.io/latest release remains `0.2.11` and this work does not publish
or tag a release.

### Replace the consumer-implemented trait

The old API allowed an implementation to override the consumer-facing methods:

```rust,ignore
impl ProtocolVerifier for Backend {
    fn verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        // A backend could accidentally skip shared checks here.
    }
}
```

Implement the lower-level hooks instead and wrap the backend in the concrete
façade:

```rust,ignore
impl ProtocolVerifierBackend for Backend {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        // Chain-specific acquisition and cryptographic verification.
    }

    fn backend_get_latest_verified_block(
        &self,
        chain: &ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        // Chain-specific latest-block evidence.
    }

    fn backend_verify_transaction_finality(
        &self,
        request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        // Chain-specific finality evidence.
    }
}

let verifier = ProtocolVerifier::try_new(backend)?;
let result = verifier.verify_chain_state(&request)?;
```

Consumers must call the façade methods. They validate capabilities and
requests before the backend hook, then validate chain/block/proof identity,
requested state-root presence/equality, provenance timestamps, trust policy,
and finality postconditions before returning success. Use
`DynProtocolVerifier` when runtime-selected backends need dynamic dispatch.

### Chain IDs and evidence

- `ChainId::from_chain(chain, network)` now derives the canonical family. Use
  `ChainId::try_from_parts` when constructing explicit parts; mismatched known
  `Chain`/`ChainFamily` pairs fail, including during deserialization.
- If a proof request includes a `ProofEnvelope`, compute and populate both
  `ProofData.evidence_hash` and `ProofEnvelope.evidence_hash` with
  `compute_evidence_binding_hash`. The envelope destination must equal the
  request's canonical chain ID.
- Validate time-sensitive requests with `validate_at`/`*_at` methods when a
  deterministic clock is required. The policy is `observed_at <= now <
  expires_at`; provenance `verified_at` must not be future-dated.

The SHA-256 binding is a structural consistency check only. It does not replace
signatures, attestations, light clients, or verifier-set proofs and must not be
described as cryptographic authenticity or production readiness.

### Why This Change?

1. **Separation of Concerns**: Protocol primitives vs. production SDK capabilities
2. **Faster Releases**: Independent release cycles for SDK vs. protocol
3. **Clear Ownership**: SDK team can iterate faster on signing primitives
4. **Production Hardening**: `conxius-enclave-sdk` has comprehensive test suites, WASM bindings, and production hardening

## Migration Steps

### Step 1: Update Dependencies

Add `conxius-enclave-sdk` to your `Cargo.toml`:

```toml
[dependencies]
conxius-enclave-sdk = "2.0.11"
lib-conxian-core = "0.2.11"
```

For Vault SDK re-exports (optional):

```toml
[dependencies]
lib-conxian-core = { version = "0.2.11", features = ["enclave"] }
```

### Step 2: Migrate VaultSDK Usage

#### Old Code (lib-conxian-core v0.2.10)

```rust
use lib_conxian_core::{VaultSDK, SigningPolicy, Wallet};

// Initialize the Wallet
let wallet = Wallet::from_private_key_hex("01".repeat(32))?;

// Create signing policy
let policy = SigningPolicy {
    max_amount_sats: 1_000_000,
    allowed_destinations: vec!["bc1q...".to_string()],
    require_biometric: true,
    timelock_blocks: 144,
};

// Initialize SDK
let sdk = VaultSDK::new(wallet, policy);

// Sign with policy enforcement
let signature = sdk.sign_with_policy("tx_id", 500_000, "bc1q...")?;
```

#### New Code (conxius-enclave-sdk)

```rust
use conxius_enclave_sdk::enclave::{
    CloudEnclave, EnclaveManager, SignRequest, SigningAlgorithm,
};

// Initialize Cloud Enclave
let enclave = CloudEnclave::new("https://vault.conxian-labs.com".to_string())?;
enclave.initialize()?;

// Create signing request
let request = SignRequest {
    algorithm: SigningAlgorithm::EcdsaSecp256k1,
    message_hash: tx_id.as_bytes().to_vec(),
    derivation_path: "m/44'/0'/0'/0/0".to_string(),
    key_id: "default".to_string(),
    taproot_tweak: None,
};

// Sign with hardware attestation
let response = enclave.sign(request)?;
println!("Signature: {}", response.signature_hex);
```

### Step 3: Migrate Policy Enforcement

The old `SigningPolicy` is replaced by custom application logic:

```rust
// Old policy enforcement
if amount_sats > policy.max_amount_sats {
    return Err("Amount exceeds limit");
}

// New: Implement your policy checks
fn validate_transaction(amount_sats: u64, destination: &str) -> Result<(), String> {
    let max_amount = 1_000_000; // Your limit
    let allowed_destinations = ["bc1q...", "bc1q..."]; // Your list
    
    if amount_sats > max_amount {
        return Err("Amount exceeds limit".to_string());
    }
    if !allowed_destinations.contains(&destination) {
        return Err("Destination not allowed".to_string());
    }
    Ok(())
}
```

### Step 4: Migrate Wallet Usage

For basic key management, use `k256` directly or `bdk`:

```rust
// Old
use lib_conxian_core::Wallet;
let wallet = Wallet::from_private_key_hex(private_key_hex)?;
let signature = wallet.sign(message);

// New: Use k256 directly
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use sha2::{Digest, Sha256};

let signing_key = SigningKey::from_slice(&hex::decode(private_key_hex)?)?;
let mut hasher = Sha256::new();
hasher.update(message.as_bytes());
let digest = hasher.finalize();
let signature: Signature = signing_key.sign(&digest);
```

### Step 5: Migrate MuSig2 Usage

Use the `musig2` crate directly or `conxius-enclave-sdk`:

```rust
// Old
use lib_conxian_core::musig2::{Musig2Participant, aggregate_public_keys};
let participant = Musig2Participant::new();

// New: Use musig2 crate
use musig2::{KeyAggContext, secp};
use secp256k1::PublicKey;

let points: Vec<secp::Point> = pubkeys.iter()
    .map(|pk| secp::Point::from(*pk))
    .collect();
let ctx = KeyAggContext::new(points)?;
let aggregated_pubkey = ctx.aggregated_pubkey();

// Or use conxius-enclave-sdk for production
use conxius_enclave_sdk::protocol::musig2::MuSig2Session;
```

### Step 6: Migrate BitVM2 Usage

```rust
// Old
use lib_conxian_core::bitvm2::{Bitvm2Orchestrator, Bitvm2Segment};
let orchestrator = Bitvm2Orchestrator::new();
let segments = orchestrator.generate_segments(state_root);

// New: Use conxius-enclave-sdk
use conxius_enclave_sdk::protocol::bitvm2::{BitVm2Orchestrator, Bitvm2Segment};
let orchestrator = BitVm2Orchestrator::new();
let segments = orchestrator.generate_segments(state_root);
```

### Step 7: Migrate Contract Bridge Usage

The `contract_bridge` module now uses `k256` directly:

```rust
// Old
use lib_conxian_core::{ContractBridge, Wallet};
let signed_call = ContractBridge::create_signed_call(&wallet, contract, function, args)?;

// New
use k256::ecdsa::SigningKey;
use lib_conxian_core::ContractBridge;

let signing_key = SigningKey::from_slice(&hex::decode(private_key_hex)?)?;
let signed_call = ContractBridge::create_signed_call(&signing_key, contract, function, args)?;
```

## Module Mapping

| Old Module | New Location | Notes |
|------------|-------------|-------|
| `VaultSDK` | `conxius_enclave_sdk::enclave::CloudEnclave` | Use for hardware signing |
| `SigningPolicy` | Custom application logic | Implement your own checks |
| `Wallet` | `k256` crate or `bdk_wallet` | For key management |
| `Musig2Participant` | `musig2::KeyAggContext` or `conxius_enclave_sdk::protocol::musig2` | BIP-327 compliant |
| `Bitvm2Orchestrator` | `conxius_enclave_sdk::protocol::bitvm2::BitVm2Orchestrator` | Full challenge support |
| `ContractBridge` | `lib_conxian_core::ContractBridge` | Updated to use k256 |

## What Stays in lib-conxian-core

These modules remain in `lib-conxian-core`:

| Module | Purpose |
|--------|---------|
| `control_model` | Trust tiers, lifecycle states, invariant validation |
| `anchoring` | State root persistence models |
| `adapters` | Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint) |
| `contract_bridge` | Clarity contract interfaces (updated to use k256) |
| `deployment` | Deployment plan and manifest types |
| `protocol` | DLC, covenants, FROST, intents, etc. |
| `bitcoin`, `stacks`, `lightning`, `rgb` | CXIP 20 modular architecture |

## Timeline

| Version | Date | Changes |
|---------|------|---------|
| v0.2.10 | 2026-07-15 | SDK marked deprecated, SDK dependency added |
| v0.2.11 | 2026-07-15 | **Breaking**: VaultSDK, Wallet, Musig2, BitVM2 removed |
| v0.3.0 | TBD | Full release with updated architecture |

## Getting Help

- **Support**: support@conxian-labs.com
- **Security**: security@conxian-labs.com
- **Documentation**: https://docs.rs/conxius-enclave-sdk
- **GitHub Issues**: https://github.com/Conxian/conxius-enclave-sdk/issues

## Changelog Summary

### Removed in v0.2.11
- `VaultSDK` struct and methods
- `SigningPolicy` struct and methods
- `Wallet` struct and methods
- `Musig2Participant` and related functions
- `Bitvm2Orchestrator` and related functions
- All deprecated re-exports

### Updated in v0.2.11
- `ContractBridge` now uses `k256::ecdsa::SigningKey` directly

### Available via `enclave` feature
- `conxius_enclave_sdk::enclave::*` - Hardware attestation, signing
- `conxius_enclave_sdk::protocol::musig2::*` - MuSig2 sessions
- `conxius_enclave_sdk::protocol::bitvm2::*` - BitVM2 orchestration
