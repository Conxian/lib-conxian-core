# `lib-conxian-core-enclave`

`lib-conxian-core-enclave` is the companion adapter crate for the exact
published `conxius-enclave-sdk =2.0.11` release. It keeps the Core crate's
default feature graph SDK-independent while providing a small, fail-closed
boundary for applications that inject an SDK `EnclaveManager`.

This crate owns only protocol-to-SDK mappings and request/response validation.
It does **not** own key custody, provider selection, attestation verification,
replay protection, networking, persistence, telemetry, or environment-specific
behavior.

## Usage

```toml
[dependencies]
lib-conxian-core-enclave = "0.1.0"
conxius-enclave-sdk = "=2.0.11"
```

The application supplies an `Arc<dyn EnclaveManager>` from the SDK's concrete
runtime and selects a Core `TrustTier`:

```rust,no_run
use std::sync::Arc;

use conxius_enclave_sdk::enclave::EnclaveManager;
use lib_conxian_core::control_model::TrustTier;
use lib_conxian_core_enclave::EnclaveSdkAdapter;

fn adapter(manager: Arc<dyn EnclaveManager>) -> lib_conxian_core_enclave::EnclaveSdkAdapter {
    EnclaveSdkAdapter::new(manager, "production-key", TrustTier::Managed)
        .expect("non-empty key identifier")
}
```

The adapter accepts only an explicit 32-byte Core `DigestAlgorithm::Sha256`
payload. It rejects Core `Message` payloads and every other digest algorithm;
it never silently hashes or relabels caller bytes. Bitcoin signing must use
`sign_digest_with_bip110_preflight`, which evaluates Core's canonical preflight
validator before invoking the injected manager.

## Capability matrix for SDK `2.0.11`

| Surface | Adapter behavior | Ownership / caveat |
| --- | --- | --- |
| Algorithms | Explicit conversion for ECDSA secp256k1, Schnorr secp256k1, and Ed25519 | The overlap is limited to the exact SDK enum; no implicit algorithm fallback exists. |
| Payloads | 32-byte Core SHA-256 digests only | The SDK request has `message_hash` but no digest-algorithm field, so messages, SHA-512, Keccak-256, and Blake2b-256 are rejected. |
| Derivation | Deterministic `m/<index>` rendering with `'` for hardened components | Core purpose metadata is preserved in the Core request/result but is not invented as an SDK path component because SDK `2.0.11` has no purpose field. |
| ECDSA response | 64-byte compact or 65-byte recoverable signatures; 33- or 65-byte public keys | Hex decoding and length checks are performed before Core response construction. |
| Schnorr response | 64-byte compact signatures; 32-, 33-, or 65-byte public keys | Provider-specific meaning of a non-x-only key remains downstream-owned. |
| Ed25519 response | 64-byte raw signatures; 32-byte public keys | No provider behavior is inferred from the enum mapping. |
| Trust policy | `Strict` requires StrongBox/CloudTEE; `Managed` and `Expedient` require TEE or stronger; `ObserverOnly` never signs | Software attestation is rejected for signing. The adapter does not verify attestation cryptography. |
| BIP-110 | Bitcoin signing requires a compliant Core preflight result before provider invocation | Transaction parsing, byte classification, serialization, and deployment state remain downstream-owned. |
| Manager boundary | Injected `Arc<dyn EnclaveManager>` and exact SDK request/response types | Lifecycle, unlock policy, replay state, provider selection, and runtime side effects remain SDK/application-owned. |

### Intentionally unsupported

- Raw message signing or implicit hashing.
- Digest algorithms not represented unambiguously by SDK `2.0.11`.
- Silent fallback to legacy signing when a stronger typed API is absent.
- Automatic Taproot tweak construction; the adapter sends `taproot_tweak: None`.
- Attestation verification, replay protection, network calls, database access,
  telemetry, or provider-specific policy.
- Claims that simulator/mock paths or the SDK's cloud simulation are production
  evidence.

## Toolchain and production posture

The effective workspace/package support floor is Rust `1.91+`, matching the
Core package metadata and CI toolchain. The SDK `2.0.11` manifest declares a
lower Rust version, but its locked dependency graph includes components that
require Rust `1.91`; the higher effective floor is therefore intentional.

The adapter is a safe shared contract surface, not a production-readiness
claim for every SDK protocol or provider. Applications must use a verified
SDK/runtime implementation, perform any required attestation verification and
approval workflow, and preserve Core's fail-closed errors before treating a
signature as usable.

## Tests

The integration tests use deterministic in-process `EnclaveManager` doubles.
They cover mapping boundaries, digest/message rejection, derivation rendering,
malformed responses, trust-tier gates, BIP-110 provider gating, and secret-safe
errors. They do not assert simulator success as production evidence.

```text
cargo test -p lib-conxian-core-enclave --locked
```
