# `lib-conxian-core-enclave`

`lib-conxian-core-enclave` is the companion adapter crate for the exact
published `conxius-enclave-sdk =2.0.11` release, which remains the latest
published SDK target. Any `2.0.12` value on unreleased upstream `main` is
metadata for that unreleased line only; this crate does not target it. The
adapter keeps the Core crate's default feature graph SDK-independent while
providing a small, fail-closed boundary for applications that inject an SDK
`EnclaveManager`.

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
| Chain/algorithm gate | Bitcoin, Liquid, Lightning, and Babylon allow secp256k1; Stacks and Ethereum allow ECDSA secp256k1; Solana allows Ed25519 | This is a concrete deny-by-default allowlist based on Core `Chain`/`ChainFamily` semantics. Other chains and pairs are rejected before the manager is called. |
| Payloads | 32-byte Core SHA-256 digests only | The SDK request has `message_hash` but no digest-algorithm field, so messages, SHA-512, Keccak-256, and Blake2b-256 are rejected. |
| Derivation | Deterministic `m/<index>` rendering with `'` for hardened components | Core purpose metadata is preserved in the Core request/result but is not invented as an SDK path component because SDK `2.0.11` has no purpose field. |
| ECDSA response | 64-byte compact or 65-byte recoverable signatures; 33- or 65-byte public keys | Hex decoding and length checks are performed before Core response construction. |
| Schnorr response | 64-byte compact signatures and exactly 32-byte x-only public keys | `EnclaveManager::get_public_key` accepts only a path and may select an algorithm from path text in SDK `2.0.11`; Schnorr public-key derivation therefore fails closed and never calls the getter. |
| Ed25519 response | 64-byte raw signatures; 32-byte public keys | No provider behavior is inferred from the enum mapping. |
| Trust policy and attestation | `Strict` requires StrongBox/CloudTEE; `Managed` and `Expedient` require TEE or stronger; `ObserverOnly` never signs | The report nonce must exactly match the forwarded 32-byte digest, and the complete opaque report/evidence is retained in the response. This layer performs request binding and level gating only; it does not cryptographically verify signatures, certificates, freshness, or hardware claims. |
| BIP-110 | Bitcoin signing requires a compliant Core preflight result before provider invocation | Transaction parsing, byte classification, serialization, and deployment state remain downstream-owned. |
| Manager boundary | Injected `Arc<dyn EnclaveManager>` and exact SDK request/response types | Lifecycle, unlock policy, replay state, provider selection, and runtime side effects remain SDK/application-owned. |

### Intentionally unsupported

- Raw message signing or implicit hashing.
- Digest algorithms not represented unambiguously by SDK `2.0.11`.
- Silent fallback to legacy signing when a stronger typed API is absent.
- Automatic Taproot tweak construction; the adapter sends `taproot_tweak: None`.
- Schnorr public-key derivation through the SDK `2.0.11` path-only getter.
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
SDK/runtime implementation, perform any required cryptographic signature and
attestation verification and approval workflow, and preserve Core's fail-closed
errors before treating a signature as usable. A successful adapter response
does not mean that the response signature or attestation has been
cryptographically verified by this layer.

## Tests

The integration tests use deterministic in-process `EnclaveManager` doubles.
They cover mapping boundaries, digest/message rejection, derivation rendering,
malformed responses, request-bound attestation evidence retention and nonce
rejection, trust-tier and chain/algorithm gates, Schnorr x-only enforcement,
BIP-110 provider gating, and secret-safe errors. They do not assert simulator
success as production evidence.

```text
cargo test -p lib-conxian-core-enclave --locked
cargo package -p lib-conxian-core-enclave --locked --allow-dirty --no-verify
```

The package check is intentionally explicit because the add-on's local Core
dependency is also declared with the compatible published requirement
`lib-conxian-core = "0.3.0"`; Cargo rewrites the path dependency to that
registry requirement in the packaged manifest. It can complete only after
Core `0.3.0` is available from the configured registry. Until then, Cargo
fails closed during dependency resolution rather than accepting an older Core
release; do not weaken the requirement to make a local package command pass.
