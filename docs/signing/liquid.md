# Liquid signing and cross-layer peg flow

## Status / support level

**Address, fee, and structural-proof primitives only.** Core identifies Liquid
as a Bitcoin-family lane and exposes a minimal adapter surface. It does not
implement a peg-in or peg-out flow, Elements transaction construction,
federation custody, confidential-asset proof verification, or a concrete
Liquid signer.

Core never holds private keys, accesses hardware, performs RPC or other
network I/O, persists state, or owns runtime retries. `Chain::Liquid` mapping to
`ChainFamily::BitcoinUtxo` is a coarse taxonomy entry, not evidence that a
Liquid signer is available.

## End-to-end flow boundary

1. The Wallet or Gateway creates a cross-layer intent and determines whether
   the operation touches Bitcoin L1, Liquid/Elements, or both.
2. Core can represent a signing request with `SigningTarget::for_chain(Chain::Liquid)`
   and can expose [`LiquidAdapter`](../../src/bitcoin/liquid_adapter.rs) address,
   fee, trust, and proof-shape checks.
3. The Enclave SDK / Wallet constructs the actual Elements or Bitcoin
   transaction and signs it only when the concrete signer advertises the exact
   chain, algorithm, operation, and address format.
4. Gateway coordinates peg/federation providers, persistence, submissions, and
   reconciliation. Nexus observes chain state and verifies evidence through its
   downstream verifier backend.
5. Core's structural proof result is an input to policy; it is not a peg
   authorization or proof of federation consensus.

## Required inputs and outputs

| Boundary | Current Core representation |
| --- | --- |
| Signing target | `SigningTarget { chain: Chain::Liquid, family: ChainFamily::BitcoinUtxo }` |
| Signing request/response | `SignRequest` → `SignResponse`, or `AddressDerivationRequest` → `AddressDerivationResponse`, only through an advertised concrete capability |
| Transaction adapter input | `TxParams { amount_sats, destination, data }` for the generic fee interface |
| Address validation | `LiquidAdapter::validate_address` accepts `ex1` or `tlq1` prefixes with length at least 39; this is not full Elements address/checksum validation |
| Fee output | `estimate_fee(&TxParams) -> 500` in the current adapter, independent of transaction shape |
| Trust policy | `trust_tier() -> TrustTier::Managed` |
| Proof input | `verify_state_proof(state_root, proof)` expects at least three colon-separated components |
| Proof output | `Ok(true)`/`Ok(false)` from structural checks; no cryptographic confidential proof or federation verification is performed |
| State root | `get_state_root() -> "liquid_merkle_root"` in the current adapter; it is not a live network read |

The generic `AddressFormat` vocabulary has no dedicated Elements variant. A
concrete capability must choose an explicitly compatible representation such as
`Generic` or a suitable Bitcoin-family format and still own Liquid network and
checksum rules.

## Ownership

| Owner | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | Liquid chain/family metadata, adapter contracts, structural address/fee/proof primitives, and UCS validation | Peg custody, federation quorum, confidential cryptography, private keys, RPC, persistence, or retries |
| Conxius Enclave SDK / Wallet | Hardware-backed custody, derivation, transaction construction, user policy, and concrete signing for explicitly supported rails | Federation orchestration, chain observation, or Core's canonical contracts |
| Gateway (`conxian-gateway`) | Cross-layer workflow orchestration, federation/provider selection, persistence, idempotency, retry/reconciliation, and submissions | Private-key custody and treating structural checks as settlement proof |
| Nexus (`conxian-nexus`) | Liquid/Bitcoin observation, proof acquisition, verifier backends, and evidence provenance | Signing, peg custody, Elements transaction building, or runtime retries |

## Retryable versus terminal failures

**Potentially retryable or reconcilable downstream failures:**

- temporary federation, node, provider, or network unavailability;
- fee-estimation or observation timeouts;
- ambiguous peg submission where Gateway must query both layers before retrying;
- signer backend failure only after the signer confirms no side effect occurred.

**Terminal for the supplied request or evidence:**

- invalid address shape or unsupported network/checksum;
- malformed or structurally incomplete proof;
- cryptographic proof, federation policy, or asset-commitment rejection;
- unsupported UCS chain, algorithm, operation, or address format;
- a confirmed failed or expired peg intent.

## Fail-closed boundaries

- `LiquidAdapter::verify_state_proof` accepting three colon-separated fields
  does not verify a block hash, Merkle path, blinded proof, asset, or
  federation signature.
- The constant state root and fixed fee are not live network evidence or a
  transaction quote.
- No peg-in/peg-out implementation exists in this adapter; do not infer one
  from the `Chain::Liquid` enum or `UniversalChainAdapter` trait.
- A concrete signer must pass the UCS capability gate for the exact Liquid
  target. Falling back to a generic Bitcoin signer is not permitted without an
  explicit downstream policy decision and compatible address/transaction
  semantics.
- Unknown or unverifiable cross-layer state blocks settlement rather than
  becoming a successful structural result.

## Current gaps / unsupported behavior

- No Liquid `UniversalChainSigner`, Elements transaction builder, PSBT/PSET
  workflow, peg state machine, or federation integration exists in Core.
- No confidential-asset range-proof, surjection-proof, or federation-signature
  verification is implemented by the current adapter.
- Address validation, fee estimation, proof checks, and state-root output are
  minimal primitives with placeholder-level semantics.
- Gateway owns all network I/O, persistence, retry, and reconciliation work.

## Source links

- [Universal signing architecture](../SIGNING_ARCHITECTURE.md)
- [UCS contract and types](../../src/signing.rs)
- [Liquid adapter](../../src/bitcoin/liquid_adapter.rs)
- [Generic adapter contract](../../src/adapters/mod.rs)
- [Chain and family mapping](../../src/control_model/trust.rs)
- [Protocol verifier ownership](../architecture/PROTOCOL_VERIFIER.md)
- [Core/Gateway boundary](../ARCHITECTURE_BOUNDARIES.md)
