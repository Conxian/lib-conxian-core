# Deterministic Integration Testing

This document describes the first core-to-downstream contract test layer for issue #182. The
layer lives entirely in `lib-conxian-core` and is intentionally deterministic, offline, and
dependency-local. It verifies that downstream-shaped requests, results, and adapter DTOs can be
serialized and rejected or accepted by the public Core facades without importing a downstream
runtime.

## Scope and ownership

The tests cover public, platform-neutral contracts:

- Universal Chain Signer (UCS) request/result serialization, capability gating, and response
  postconditions.
- ProtocolVerifier proof, provenance, trust-policy, finality, and fixed-clock validation.
- BIP-110 preflight boundary measurements, ordered findings, unsupported contexts, and missing
  measurement fail-closed behavior.
- Structural DTO and rollout behavior for representative Bitcoin/Babylon or Liquid, Stacks/sBTC,
  RGB, and DLC surfaces.

The test doubles in `tests/support/` are test-only backends. They do not replace the production
facades or claim cryptographic proof verification, transaction parsing, custody, chain observation,
RPC behavior, persistence, or provider integration. The real `UniversalChainSigner`,
`ProtocolVerifier`, and `Bip110PreflightValidator` remain the validation authorities.

Core owns canonical types, versioned contracts, invariant validation, and deterministic finding
order. SDK and Wallet code own key custody, transaction construction, serialization/classification,
user approval, and concrete signing. Nexus owns observation, proof acquisition, and verifier
backends. Gateway owns orchestration, persistence, routing, retries, and external side effects.

## Enclave SDK companion tests

The workspace member `lib-conxian-core-enclave` is tested separately against the
exact published `conxius-enclave-sdk =2.0.11` API. Its integration tests inject
a deterministic in-process `EnclaveManager` double and verify the adapter
boundary rather than a simulator or provider implementation. Coverage includes:

- all three exact Core/SDK algorithm mappings;
- derivation-path rendering at root, hardened, `u32::MAX`, and component-count
  boundaries;
- byte-preserving SHA-256 digest extraction plus message and unsupported-digest
  rejection;
- SDK request construction and public response shape validation;
- malformed hex, signature/key lengths, missing/invalid attestations, exact
  attestation request binding, evidence retention, and secret-safe provider
  errors;
- `Strict`, `Managed`, `Expedient`, and `ObserverOnly` trust-policy behavior;
- `ObserverOnly` never invokes the provider;
- the deny-by-default chain/algorithm capability gate, including invalid-pair
  zero-call assertions and the Schnorr getter fail-closed rule; and
- BIP-110 rejection before provider invocation, plus inclusive compliant
  boundary values.

The tests do not verify hardware, cryptographic attestation, simulator success,
network calls, replay state, persistence, or production provider readiness.
Those behaviors remain SDK/downstream responsibilities.

## Fixture policy

Golden JSON files are under [`../tests/fixtures/`](../tests/fixtures/) and are indexed by
`manifest.json`. The manifest requires:

- fixture schema version `1`;
- package `lib-conxian-core` version `0.3.1`;
- UCS API version `1`;
- BIP-110 preflight API version `1`;
- ProtocolVerifier evidence-binding version `1` and domain
  `lib-conxian-core/protocol-verifier/evidence-binding`.

Fixtures use fixed RFC3339 timestamps, public or dummy byte arrays, stable IDs, and opaque
structural proof strings. They must not contain private keys, seeds, custody material, secrets,
RPC URLs, network credentials, environment-specific branches, or real production evidence.

Every manifest-indexed fixture file has a schema version and `cases` array. Tests load every file
listed by the manifest, reject unlisted cases-based files, compare all case IDs against the
manifest, and exercise JSON deserialize/serialize/deserialize structural round trips. The legacy
boundary fixtures `adapter_contracts.json`, `bip110_preflight.json`, `signing_boundary.json`, and
`verifier_boundary.json` are intentionally outside that manifest inventory and are consumed
separately by `core_to_downstream_integration.rs`. A new cases-based fixture must be added to the
manifest in the same change.

The BIP-110 fixtures represent classified measurements supplied by a transaction-aware caller;
they do not turn Core into a transaction parser or consensus validator. The Taproot control-block
fixture uses the current inclusive limit of `257` bytes. `258` is a violation. The ordinary limits
remain 256-byte pushdata and witness elements, 83-byte OP_RETURN ScriptPubKeys, and 34-byte
non-OP_RETURN ScriptPubKeys.

Evidence envelopes are used for deterministic stale and policy-blocked rejection paths. Avoid
long-lived success fixtures that require ambient-clock evidence-binding computation until a fixed
clock equivalent of the hash helper exists; use the public `*_at` validation APIs for all
time-sensitive assertions.

## Compatibility and downstream adoption

The exact pins represented by this checkpoint are:

| Surface | Current checkpoint |
| --- | --- |
| Core package | `lib-conxian-core` `0.3.1` |
| UCS | API version `1` |
| BIP-110 preflight | API version `1` |
| ProtocolVerifier evidence binding | version `1`, domain `lib-conxian-core/protocol-verifier/evidence-binding` |
| Optional SDK dependency in Core manifest | `conxius-enclave-sdk` exact `2.0.11` |
| Companion adapter workspace member | `lib-conxian-core-enclave` against exact SDK `2.0.11`; default features remain minimal |
| SDK main line | `2.0.12` is metadata on unreleased upstream `main`; the latest published target remains exact `2.0.11` |
| Nexus | default-branch `main` [`Cargo.toml`](https://github.com/Conxian/conxian-nexus/blob/main/Cargo.toml) currently pins `lib-conxian-core` to git revision `3b091d2700d840514427e4190c40d631b6d8132c`; this checkpoint does not change that downstream pin |
| Gateway | local Core crate integration; no cross-repository dependency is added here |
| Wallet | TypeScript boundary; no Rust runtime dependency is added here |

The current `Conxian/conxian-nexus` default branch is `main`, and its root `Cargo.toml` currently
contains the exact `lib-conxian-core` revision pin
`3b091d2700d840514427e4190c40d631b6d8132c`. This is a verified downstream manifest status, not
evidence that Nexus runtime behavior, downstream CI, or every fixture has adopted this checkpoint.
The optional direct `enclave` feature remains available for compatibility, but
the companion adapter is the tested Core/SDK boundary. The effective workspace
floor is Rust `1.94.1+`; the SDK `2.0.11` manifest's lower declaration does not
lower that floor because its locked Alloy graph requires Rust `1.94.1`. The tests
run with default features for Core and do not enable simulator/mock/dev bypasses.

This document describes the Core-only fixture layer. Direct compile and
serde-boundary evidence against the exact SDK release is intentionally kept in
the separate opt-in [`SDK v2.0.11 compatibility evidence`](COMPATIBILITY.md#opt-in-sdk-v2011-compatibility-evidence)
harness. That harness does not change the ownership or production-readiness
claims of these deterministic Core fixtures.

Downstream CI fan-out is deliberately deferred. SDK, Nexus, Gateway, and Wallet should adopt the
contract versions and pins explicitly before repository-to-repository CI is added. Until then,
this repository validates only its local public contract surface and does not imply that a
downstream consumer currently enforces every fixture or finding.

## Local commands

Run from the repository root:

```text
cargo fmt --all -- --check
cargo test -p lib-conxian-core-enclave --locked
cargo test --test core_to_downstream_integration --locked
cargo test --test golden_serialization --locked
cargo test --test deterministic_contracts --locked
cargo test --test adapter_conformance --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo package -p lib-conxian-core-enclave --locked --allow-dirty --no-verify
```

The add-on package check is a release/package dry-run for the workspace
member. Its manifest keeps the local path for workspace builds and declares
the compatible published Core requirement `lib-conxian-core = "0.3.1"` so Cargo
can produce a registry-ready package manifest. The check requires Core `0.3.1`
to be available from the configured registry; before that publication Cargo
must reject resolution rather than silently selecting the older published
`0.2.11` Core release.

The integration tests are credential-free and do not require a node, RPC endpoint, database,
hardware-backed signer, enclave, custody service, or environment-specific configuration.
