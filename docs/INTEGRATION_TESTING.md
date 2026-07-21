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

## Fixture policy

Golden JSON files are under [`../tests/fixtures/`](../tests/fixtures/) and are indexed by
`manifest.json`. The manifest requires:

- fixture schema version `1`;
- package `lib-conxian-core` version `0.2.12`;
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
| Core package | `lib-conxian-core` `0.2.12` |
| UCS | API version `1` |
| BIP-110 preflight | API version `1` |
| ProtocolVerifier evidence binding | version `1`, domain `lib-conxian-core/protocol-verifier/evidence-binding` |
| Optional SDK dependency in Core manifest | `conxius-enclave-sdk` `2.0.11` |
| SDK main line | `conxius-enclave-sdk` `2.0.12`; not imported by this test layer |
| Nexus | default-branch `main` [`Cargo.toml`](https://github.com/Conxian/conxian-nexus/blob/main/Cargo.toml) currently pins `lib-conxian-core` to git revision `3b091d2700d840514427e4190c40d631b6d8132c`; this checkpoint does not change that downstream pin |
| Gateway | local Core crate integration; no cross-repository dependency is added here |
| Wallet | TypeScript boundary; no Rust runtime dependency is added here |

The current `Conxian/conxian-nexus` default branch is `main`, and its root `Cargo.toml` currently
contains the exact `lib-conxian-core` revision pin
`3b091d2700d840514427e4190c40d631b6d8132c`. This is a verified downstream manifest status, not
evidence that Nexus runtime behavior, downstream CI, or every fixture has adopted this checkpoint.
The optional `enclave` feature
has a known downstream MSRV/dependency mismatch and is not required by this test layer. The tests
run against default features and do not require `--all-features`.

Downstream CI fan-out is deliberately deferred. SDK, Nexus, Gateway, and Wallet should adopt the
contract versions and pins explicitly before repository-to-repository CI is added. Until then,
this repository validates only its local public contract surface and does not imply that a
downstream consumer currently enforces every fixture or finding.

## Local commands

Run from the repository root:

```text
cargo fmt --all -- --check
cargo test --test golden_serialization --locked
cargo test --test deterministic_contracts --locked
cargo test --test adapter_conformance --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The integration tests are credential-free and do not require a node, RPC endpoint, database,
hardware-backed signer, enclave, custody service, or environment-specific configuration.
