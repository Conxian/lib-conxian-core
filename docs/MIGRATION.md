# Migration Guide: lib-conxian-core v0.2.x → v0.3.0

> **Important**: This document describes the migration from deprecated VaultSDK to the production `conxius-enclave-sdk`.

## Overview

Starting with v0.3.0, `lib-conxian-core` will no longer include Vault SDK functionality (hardware-backed signing, attestation, policy enforcement). This functionality has moved to the production [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate.

### Why This Change?

1. **Separation of Concerns**: Protocol primitives vs. production SDK capabilities
2. **Faster Releases**: Independent release cycles for SDK vs. protocol
3. **Clear Ownership**: SDK team can iterate faster on signing primitives
4. **Production Hardening**: `conxius-enclave-sdk` has comprehensive test suites, WASM bindings, and production hardening

## Migration Steps

### Step 1: Add the New Dependency

Replace `lib-conxian-core` with `conxius-enclave-sdk`:

```toml
# Old (deprecated)
[dependencies]
lib-conxian-core = "0.2.10"

# New
[dependencies]
conxius-enclave-sdk = "2.0.11"
lib-conxian-core = { version = "0.3.0", features = ["enclave"] }  # Optional: for protocol primitives only
```

### Step 2: Migrate VaultSDK Usage

#### Old Code (lib-conxian-core)

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

## Module Mapping

| Old Module | New Location | Notes |
|------------|-------------|-------|
| `VaultSDK` | `conxius_enclave_sdk::enclave::CloudEnclave` | Use for hardware signing |
| `SigningPolicy` | Custom application logic | Implement your own checks |
| `Wallet` | `k256` crate or `bdk_wallet` | For key management |
| `Musig2Participant` | `conxius_enclave_sdk::protocol::musig2::MuSig2Session` | BIP-327 compliant |
| `Bitvm2Orchestrator` | `conxius_enclave_sdk::protocol::bitvm2::BitVm2Orchestrator` | Full challenge support |

## What Stays in lib-conxian-core

These modules remain in `lib-conxian-core`:

| Module | Purpose |
|--------|---------|
| `control_model` | Trust tiers, lifecycle states, invariant validation |
| `anchoring` | State root persistence models |
| `adapters` | Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon) |
| `contract_bridge` | Clarity contract interfaces |
| `musig2` | **Deprecated** - Use SDK |
| `bitvm2` | **Deprecated** - Use SDK |

## Timeline

| Version | Date | Changes |
|---------|------|---------|
| v0.2.10 | 2026-07-15 | SDK marked deprecated, SDK dependency added |
| v0.2.11 | TBD | Warning added to compiler output |
| v0.3.0 | TBD | **Breaking**: VaultSDK removed, use conxius-enclave-sdk |

## Getting Help

- **Support**: support@conxian-labs.com
- **Security**: security@conxian-labs.com
- **Documentation**: https://docs.rs/conxius-enclave-sdk
- **GitHub Issues**: https://github.com/Conxian/conxius-enclave-sdk/issues

## Changelog Summary

### Removed in v0.3.0
- `VaultSDK` struct and methods
- `SigningPolicy` struct and methods
- `Wallet::sign()` - Use k256 directly
- `Wallet::from_private_key_hex()` - Use k256 directly

### Deprecated (v0.2.x)
- All above items emit deprecation warnings

### Available via `enclave` feature
- `conxius_enclave_sdk::enclave::*` - Hardware attestation, signing
- `conxius_enclave_sdk::protocol::musig2::*` - MuSig2 sessions
- `conxius_enclave_sdk::protocol::bitvm2::*` - BitVM2 orchestration
