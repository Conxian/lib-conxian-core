# Bitcoin signing flow

## Status / support level

**Protocol contract and preflight boundary only.** Core represents Bitcoin
signing requests, capability gates, public signing responses, and BIP-110 size
metadata. It does not provide a concrete signer or a complete Bitcoin
transaction workflow. A `Chain::Bitcoin` entry or a
`ChainSigningCapability` declaration is vocabulary/advertisement only; it does
not imply that an implementation is available.

Core never holds private keys, accesses hardware, performs RPC or other
network I/O, persists state, or owns runtime retries. The concrete signer and
wallet flow remain downstream.

## End-to-end flow boundary

1. The Wallet or Gateway creates a chain-specific message, digest, or already
   constructed Bitcoin transaction in the downstream signing layer. Core does
   not construct transactions or PSBTs.
2. A transaction-aware downstream adapter classifies the Bitcoin surfaces and
   supplies a [`Bip110TransactionShape`](../../src/control_model/bip110.rs).
   `Bip110Compliance` checks the supplied pushdata, ScriptPubKey, and applicable
   witness sizes. The adapter, SDK, Wallet, Gateway, or Nexus owns the decision
   to reject or route the transaction.
3. A concrete signer advertises only the targets, algorithms, operations, and
   address formats it actually implements through
   [`SignerCapabilities`](../../src/signing.rs).
4. [`UniversalChainSigner::sign`](../../src/signing.rs) validates the target,
   payload, derivation metadata, and capability before invoking the concrete
   `sign_impl` hook. The signer returns public verification metadata in a
   `SignResponse`.
5. The Gateway coordinates policy, persistence, submission, and reconciliation.
   Nexus observes Bitcoin state and supplies separately verified evidence. Core
   does not broadcast or poll.

The BIP-110 handoff is a size contract, not a transaction parser or consensus
validator. The adapter must classify every relevant occurrence before calling
Core; unsupported or unclassified Taproot/script context must not be silently
treated as compliant.

## Required inputs and outputs

| Boundary | Current Core representation |
| --- | --- |
| Signing target | `SigningTarget { chain: Chain::Bitcoin, family: ChainFamily::BitcoinUtxo }` |
| Signing request | `SignRequest { target, algorithm, payload, derivation }` |
| Payload | `SigningPayload::Message { bytes }` or `SigningPayload::Digest { algorithm, bytes }`; hashing and Bitcoin domain encoding are caller-owned |
| Derivation metadata | `DerivationContext { path: DerivationPath, purpose: DerivationPurpose }`; it carries no seed, key, share, or hardware handle |
| Address derivation | `AddressDerivationRequest` → `AddressDerivationResponse { verification_key, address, derivation }` |
| Signing response | `SignResponse { signature, verification_key, address, derivation }`; all returned key material is public |
| Verification request | `VerificationRequest { target, algorithm, payload, signature, verification_key, address }` |
| Verification result | `VerificationResult { valid, target, algorithm }`; `valid: false` is a valid negative result, not a retry signal |
| BIP-110 preflight | `Bip110TransactionShape { pushdata_sizes_bytes, op_return_script_pubkey_sizes_bytes, non_op_return_script_pubkey_sizes_bytes, witness_element_sizes_bytes }` |

For Bitcoin addresses, the signing contract can advertise
`AddressFormat::BitcoinBase58` or `AddressFormat::BitcoinBech32`, but the
concrete signer remains responsible for network, checksum, script, and address
policy validation.

## Ownership

| Owner | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | UCS DTOs, capability gating, request/response invariants, BIP-110 size validation, and protocol contracts | Private keys, hardware, transaction construction, Taproot execution, RPC, persistence, broadcast, or retries |
| Conxius Enclave SDK / Wallet | Hardware-backed key custody, derivation, concrete UCS implementation, transaction/PSBT construction, user policy, and signing approval | Gateway routing, chain observation, or Core's canonical DTO definitions |
| Gateway (`conxian-gateway`) | Runtime orchestration, provider selection, persistence, idempotency, bounded retries, network submission, and reconciliation | Private-key custody and replacement of Core capability checks |
| Nexus (`conxian-nexus`) | Bitcoin observation, header/proof acquisition, and evidence consumed by routing or finality decisions | Signing keys, transaction construction, or runtime signing retries |

## Retryable versus terminal failures

**Retry or reconcile only when the downstream owner can prove it is safe:**

- temporary provider, node, fee-estimation, or network failure;
- a signer backend timeout where the concrete signer can establish that no
  signature was produced and the request remains idempotent;
- an observation/finality timeout handled by Gateway or Nexus.

`SigningError::BackendFailure` does not itself declare retryability. Core cannot
decide whether a hardware operation partially completed.

**Terminal for the current request:**

- `InvalidTarget`, `UnsupportedChain`, `UnsupportedAlgorithm`, or
  `UnsupportedOperation`;
- invalid payload, derivation path, address, signature, verification key,
  request, or signer response;
- any BIP-110 violation after the adapter has classified the relevant bytes;
- an unsupported or unclassified Taproot/script context;
- `VerificationResult { valid: false, .. }`, which requires rejection or a new
  request rather than blind retry.

## Fail-closed boundaries

- `SignerCapabilities::require` must pass for the exact chain/family,
  algorithm, and operation before a signer hook is called.
- `SigningTarget::validate` rejects an inconsistent chain/family pair.
- Payload, derivation, address, response, and verification metadata are
  validated before success is returned.
- A downstream preflight must not claim BIP-110 compliance when it cannot
  classify a witness, Taproot, or script context.
- BIP-110 compliance is an explicit caller choice; the default
  `Bip110Compliance` value is disabled even though its canonical limits are
  present.
- No signature, key, address, or transaction should be submitted merely
  because the chain enum exists.

## Taproot, Miniscript, construction, and BIP-322 limits

Core does not implement the complete BIP-341/BIP-342 boundary. A downstream
parser/interpreter must own Taproot key-path versus script-path selection,
annex detection, control-block shape and commitment checks, undefined witness
or leaf versions, `OP_SUCCESSx`, and executed `OP_IF`/`OP_NOTIF` behavior.
Miniscript compilation, satisfaction construction, and policy/tree validation
are also outside the current Core contract. The separate handoff is tracked at
the known canonical issue [#178](https://github.com/Conxian/lib-conxian-core/issues/178).

[`Bip322Bridge`](../../src/bitcoin/bip322.rs) is not a production authorization
boundary: its current helper can accept a bech32-looking address when parsing
fails, constructs the BIP-322 transaction shapes but does not execute script
signature validation, and ultimately treats a non-empty witness as success.
Callers must not use that boolean as proof of a valid Bitcoin signature.

## Current gaps / unsupported behavior

- No concrete Bitcoin `UniversalChainSigner` implementation exists in Core.
- Core does not construct transactions or PSBTs, apply network-specific fee or
  UTXO policy, submit transactions, or manage replacements.
- Core does not implement BIP-341/BIP-342 cryptography or execution, a
  Miniscript compiler, or a complete BIP-322 verifier.
- BIP-110 fields validate supplied byte metadata only; they do not infer
  activation, deployment, UTXO grandfathering, or script exceptions.

## Source links

- [Universal signing architecture](../SIGNING_ARCHITECTURE.md)
- [UCS contract and types](../../src/signing.rs)
- [Bitcoin BIP-322 helper](../../src/bitcoin/bip322.rs)
- [BIP-110 Core facade](../../src/control_model/bip110.rs)
- [BIP-110 alignment and downstream handoff](../BIP110_ALIGNMENT.md)
- [Core/Gateway boundary](../ARCHITECTURE_BOUNDARIES.md)
- [Protocol verifier ownership](../architecture/PROTOCOL_VERIFIER.md)
