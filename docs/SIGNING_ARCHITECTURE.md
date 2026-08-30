# Universal Chain Signing Architecture

`lib-conxian-core` defines the platform-neutral contract for chain signing. It
does not hold private keys, create keys, access hardware, call RPC providers, or
coordinate runtime workflows. Concrete implementations advertise the subset of
the contract they support through `SignerCapabilities`; unsupported chains,
algorithms, and operations fail closed.

## Core contract

The `signing` module provides:

- `UniversalChainSigner`: capability discovery plus validated signing,
  address-derivation, and signature-verification entry points.
- `SignRequest` and `AddressDerivationRequest`: explicit chain/family,
  algorithm, payload, and derivation context.
- `VerificationRequest`: complete verification input containing the payload or
  digest, signature, public verification key, and optional expected address.
- `SignResponse`, `AddressDerivationResponse`, and `VerificationResult`:
  public verification metadata and a positive/negative verification result.
- `ChainSigningCapability` and `SignerCapabilities`: versioned declarations of
  supported targets, algorithms, operations, and address formats.
- `SigningError`: structured, serializable, secret-safe failures. Error values
  never include payload bytes, signatures, addresses, or key material.

`SigningPayload` is either a non-empty message or a digest with an explicit
digest algorithm and length. No chain-domain hashing or encoding is implicit.
`DerivationContext` contains only structured path and purpose metadata; it never
contains a seed, private key, share, or enclave handle.

## Exact published enclave SDK companion

The workspace member [`lib-conxian-core-enclave`](../addons/lib-conxian-core-enclave/)
is the cycle-safe adapter boundary for the exact published
`conxius-enclave-sdk =2.0.17` API. It depends on Core and the SDK directly while
Core's default features remain SDK-independent. Applications inject an
`Arc<dyn EnclaveManager>`; the adapter does not construct a provider or own its
lifecycle. The published SDK `2.0.17` remains standalone and does not depend on
Core; any future reverse edge requires a separate dependency-graph review.

| Contract surface | Implemented behavior | Fail-closed boundary |
| --- | --- | --- |
| Algorithm mapping | ECDSA secp256k1, Schnorr secp256k1, and Ed25519 map explicitly in both directions | No fallback algorithm or provider-specific enum is inferred. |
| Payload mapping | Only an explicit 32-byte Core SHA-256 digest is copied into SDK `message_hash` | Core messages and SHA-512, Keccak-256, and Blake2b-256 digests are rejected because SDK `2.0.17` carries no digest-algorithm discriminator. |
| Derivation | Structured Core indices render deterministically as `m/<index>` with `'` for hardened components | Core purpose is not invented as a path component because the SDK request has no purpose field. |
| Public response | Exact SDK hex fields map to Core signatures and public verification keys after length checks | Malformed hex, unsupported lengths, missing fields, and invalid attestation JSON produce typed errors without raw provider values. |
| Trust policy | `Strict` requires hardware-backed StrongBox/CloudTEE; `Managed` and `Expedient` require TEE or stronger; `ObserverOnly` cannot sign | Software attestation is never accepted for signing; cryptographic attestation verification remains SDK/downstream-owned. |
| Rail/network policy | The adapter owns explicit Core-to-SDK rail checks and a fallible wire mapping for SDK `Network::{Mainnet,Testnet,Devnet}` | Weaker observed rail tiers and unknown serialized values fail closed; SDK `T4` is observation-only and is never a sign-capable Core `ObserverOnly` mapping. |
| Replay/idempotency binding | A typed adapter binding commits Core `SignedEnvelopeDescriptor` idempotency key/sequence and the original digest to the digest sent to SDK `2.0.17` | Missing or mismatched bindings are rejected before provider invocation. Duplicate storage, replay cache TTL, and persistence remain SDK/higher-runtime-owned. |
| Bitcoin gate | Core's canonical `validate_bip110_preflight` runs before `EnclaveManager::sign` | Non-compliant, missing, unsupported, or mismatched preflight results cannot reach the provider. The adapter does not parse or serialize Bitcoin transactions. |

The adapter supports the safe shared contract surface only. It does not claim
that every SDK protocol/provider is production-ready, and it does not silently
fall back to legacy signing when the exact typed SDK API cannot represent a
request. Simulator/mock/dev paths are not enabled by default and are not
production evidence.

## Existing chain-family mapping

The contract reuses the existing `Chain` and `ChainFamily` models. Bitcoin,
Stacks, Liquid, Babylon, and Lightning use the existing `BitcoinUtxo` family;
their concrete address formats and operation support remain capability-level
decisions. Ethereum, Base, Arbitrum, Optimism, Polygon, and the existing EVM
lanes use `Evm`. Solana/Eclipse use `SolanaSvm`, Cosmos lanes use `CosmosIbc`,
Move lanes use `Move`, and Polkadot/Kusama use `Substrate`.

RGB and DLC flows are Bitcoin-adjacent protocol operations. Their concrete
adapters should select the Bitcoin-family target and advertise only the
algorithms and operations they can safely implement. The core contract does not
claim that any of these runtime adapters exist.

## Request examples

The examples below construct protocol requests only. They do not imply that
`lib-conxian-core` can sign on those networks by itself.

```rust
use lib_conxian_core::control_model::Chain;
use lib_conxian_core::signing::{
    DerivationContext, DerivationPath, DerivationPurpose, SignRequest,
    SigningAlgorithm, SigningPayload, SigningTarget,
};

let bitcoin = SignRequest::new(
    SigningTarget::for_chain(Chain::Bitcoin),
    SigningAlgorithm::SchnorrSecp256k1,
    SigningPayload::message(b"bitcoin message".to_vec()),
    DerivationContext::new(DerivationPath::root(), DerivationPurpose::MessageSigning),
);

let stacks = SignRequest::new(
    SigningTarget::for_chain(Chain::Stacks),
    SigningAlgorithm::EcdsaSecp256k1,
    SigningPayload::message(b"stacks message".to_vec()),
    DerivationContext::new(DerivationPath::root(), DerivationPurpose::MessageSigning),
);

let ethereum = SignRequest::new(
    SigningTarget::for_chain(Chain::Ethereum),
    SigningAlgorithm::EcdsaSecp256k1,
    SigningPayload::message(b"eip-191 bytes supplied by the caller".to_vec()),
    DerivationContext::new(DerivationPath::root(), DerivationPurpose::MessageSigning),
);

let solana = SignRequest::new(
    SigningTarget::for_chain(Chain::Solana),
    SigningAlgorithm::Ed25519,
    SigningPayload::message(b"solana message".to_vec()),
    DerivationContext::new(DerivationPath::root(), DerivationPurpose::MessageSigning),
);
```

For verification, callers must pass the same payload representation used for
signing, the signature, and the public verification key. An expected address
may be included to bind the check to a chain address; `verify_signature` does
not accept an underspecified `(chain, signature)` pair.

## Ownership boundaries

| Layer | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | Canonical DTOs, capability checks, validation, and the `UniversalChainSigner` contract | Private keys, key generation, hardware, RPC, persistence, retries, or provider behavior |
| Enclave/Wallet SDK | Hardware-backed custody, key derivation, signing backends, attestation, and concrete `UniversalChainSigner` implementations | Core dependency, gateway routing, and cross-provider orchestration; published SDK `2.0.17` remains standalone |
| Core/SDK companion adapter | Exact Core/SDK mapping, adapter-owned rail/network policy, typed replay binding, typed request/response boundary, trust gates, and Core-first BIP-110 preflight | Provider runtime, attestation verification, replay storage/cache TTL, networking, persistence, telemetry, or environment-specific behavior |
| Gateway | Runtime coordination, policy/workflow enforcement, provider selection, retries, persistence, and external side effects | Canonical signing DTO definitions and private-key custody |
| Nexus | Chain observation, proof/state verification, and evidence used by higher-level routing | Signing key custody or core runtime orchestration |

The capability declaration is the source of truth for runtime support. Adding a
chain to the `Chain` enum or constructing a request does not claim that a signer
implementation supports it.
