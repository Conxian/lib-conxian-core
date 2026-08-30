# `lib-conxian-core-enclave`

`lib-conxian-core-enclave` is the companion adapter crate for the
`conxius-enclave-sdk` `v2.0.17` release (Git tag `v2.0.17`, published as
`2.0.17` on crates.io). The adapter keeps the Core crate's default feature
graph SDK-independent while providing a small, fail-closed boundary for
applications that inject an SDK `EnclaveManager`.

This crate owns only protocol-to-SDK mappings and request/response validation.
It does **not** own key custody, provider selection, attestation verification,
replay-cache storage or TTLs, networking, persistence, telemetry, or
environment-specific behavior. The published SDK `2.0.17` remains standalone
and does not depend on Core; this companion adapter is the only layer here that
depends on both Core and SDK. A future SDK-to-Core edge requires a separate
graph review.

## Usage

```toml
[dependencies]
lib-conxian-core-enclave = "0.1.0"
conxius-enclave-sdk = "=2.0.17"
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
validator before invoking the injected manager. Every signing entry point now
requires the canonical Core `SignedEnvelopeDescriptor` plus an adapter-owned
`RequestPolicyContext`. The adapter derives the `ReplayBinding` internally and
sends its bound digest to the SDK; callers cannot provide a binding DTO as
signing authority.

## Pre-release signing API

This addon is pre-release. The signature changes in this release are
intentional: consumers pass the canonical descriptor and request policy
context, while the provider signs a domain-separated digest bound to the
descriptor, original digest, network policy, and rail trust evidence. The
returned `ReplayBinding` is response evidence with private fields and
read-only accessors; it is not accepted by any signing method. The colon-joined
Core `idempotency_key()` remains a display value only and is not cryptographic
input.

## Capability matrix for SDK `2.0.17`

| Surface | Adapter behavior | Ownership / caveat |
| --- | --- | --- |
| Algorithms | Explicit conversion for ECDSA secp256k1, Schnorr secp256k1, and Ed25519 | The overlap is limited to the exact SDK enum; no implicit algorithm fallback exists. |
| Chain/algorithm gate | Bitcoin, Liquid, Lightning, and Babylon allow secp256k1; Stacks and Ethereum allow ECDSA secp256k1; Solana allows Ed25519 | This is a concrete deny-by-default allowlist based on Core `Chain`/`ChainFamily` semantics. Other chains and pairs are rejected before the manager is called. |
| Payloads | 32-byte Core SHA-256 digests only | The SDK request has `message_hash` but no digest-algorithm field, so messages, SHA-512, Keccak-256, and Blake2b-256 are rejected. |
| Derivation | Deterministic `m/<index>` rendering with `'` for hardened components; only ECDSA secp256k1 public-key derivation may use the SDK getter | Core purpose metadata is preserved in the Core request/result but is not invented as an SDK path component because SDK `2.0.17` has no purpose field. Schnorr and Ed25519 public-key derivation are unsupported because the getter is algorithm-agnostic; signing response validation remains separate. |
| ECDSA response | 64-byte compact or 65-byte recoverable signatures; 33- or 65-byte public keys | Hex decoding and length checks are performed before Core response construction. |
| Schnorr response | 64-byte compact signatures and exactly 32-byte x-only public keys | `EnclaveManager::get_public_key` accepts only a path and is algorithm-agnostic in SDK `2.0.17`; Schnorr public-key derivation therefore fails closed and never calls the getter. Signing response validation remains separate. |
| Ed25519 response | 64-byte raw signatures; 32-byte public keys | SDK `2.0.17`'s getter is also algorithm-agnostic, so Ed25519 public-key derivation fails closed and never calls the getter. Signing response validation remains separate. |
| Trust policy and attestation | `Strict` requires StrongBox/CloudTEE; `Managed` and `Expedient` require TEE or stronger; `ObserverOnly` never signs | Custom/deserialized policies are validated against the canonical Core floor. The report nonce must exactly match the adapter-bound SDK digest, and the complete opaque report/evidence is retained in the response. This layer performs request binding and level gating only; it does not cryptographically verify signatures, certificates, freshness, or hardware claims. |
| Rail/network policy | Every signing call requires `RequestPolicyContext`, which maps the explicit SDK network and observed rail tier against the adapter's Core `TrustPolicy` | Weaker observed tiers, tier mismatches, ObserverOnly signing, and unknown rail/network values fail closed. SDK T4 is observation-only; URLs/configuration remain outside Core. |
| Replay/idempotency binding | `ReplayBinding` commits descriptor fields (`publisher`, `event_id`, `sequence`, `payload_hash`, ordered commitments), the original digest, network, and rail policy using domain separation and length prefixes | The adapter derives the binding internally before `EnclaveManager::sign`; delimiter-collision descriptors therefore remain distinct. Duplicate detection, storage, and cache TTL remain SDK/higher-runtime-owned; this crate has no process-global replay state. |
| BIP-110 | Bitcoin signing requires a compliant Core preflight result before provider invocation | Transaction parsing, byte classification, serialization, and deployment state remain downstream-owned. |
| Manager boundary | Injected `Arc<dyn EnclaveManager>` and exact SDK request/response types | Lifecycle, unlock policy, replay storage/cache TTL, provider selection, and runtime side effects remain SDK/application-owned. |

### Intentionally unsupported

- Raw message signing or implicit hashing.
- Digest algorithms not represented unambiguously by SDK `2.0.17`.
- Silent fallback to legacy signing when a stronger typed API is absent.
- Automatic Taproot tweak construction; the adapter sends `taproot_tweak: None`.
- Schnorr and Ed25519 public-key derivation through the SDK `2.0.17`
  algorithm-agnostic path-only getter. Signing response validation for both
  algorithms remains a separate supported boundary.
- Attestation verification, replay storage, network calls, database access,
  telemetry, provider-specific policy, or runtime URL/configuration.
- Claims that simulator/mock paths or the SDK's cloud simulation are production
  evidence.

## Toolchain and production posture

The effective workspace/package support floor is Rust `1.97.1+`, matching the
Core package metadata and CI toolchain. This companion adapter intentionally
continues to target the published SDK `2.0.17`; its lower manifest declaration
does not lower the support floor of the current Core workspace.

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
rejection, trust-tier and chain/algorithm gates, rail/network downgrade and
unknown-value rejection, descriptor-derived replay-binding collision and
forgery resistance, DTO/error serde including unknown-field rejection, Schnorr
x-only enforcement, BIP-110 provider gating, and secret-safe errors.
They do not assert simulator success as production evidence.

```text
cargo test -p lib-conxian-core-enclave --locked
cargo package -p lib-conxian-core-enclave --locked --allow-dirty --no-verify
```

The package check is intentionally explicit because the add-on's local Core
dependency is also declared with the compatible published requirement
`lib-conxian-core = "0.3.1"`; Cargo rewrites the path dependency to that
registry requirement in the packaged manifest. It can complete only after
Core `0.3.1` is available from the configured registry. Until then, Cargo
fails closed during dependency resolution rather than accepting an older Core
release; do not weaken the requirement to make a local package command pass.

The release workflow therefore publishes `lib-conxian-core` first, waits for
the exact Core version to be visible through crates.io and its index, runs the
add-on package dry-run, and publishes `lib-conxian-core-enclave` only after that
dry-run resolves the registry dependency. A manual dry-run before Core is
published verifies the Core package only; the add-on dry-run is intentionally
performed in the publish path after Core propagation rather than hiding this
external ordering prerequisite.
