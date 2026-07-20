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
| Enclave/Wallet SDK | Hardware-backed custody, key derivation, signing backends, attestation, and concrete `UniversalChainSigner` implementations | Gateway routing and cross-provider orchestration |
| Gateway | Runtime coordination, policy/workflow enforcement, provider selection, retries, persistence, and external side effects | Canonical signing DTO definitions and private-key custody |
| Nexus | Chain observation, proof/state verification, and evidence used by higher-level routing | Signing key custody or core runtime orchestration |

The capability declaration is the source of truth for runtime support. Adding a
chain to the `Chain` enum or constructing a request does not claim that a signer
implementation supports it.
