# Bitcoin signing flow

## Scope and current support status

This guide covers Bitcoin message, digest, and transaction-signing handoffs in
the current Core contract. Core supplies platform-neutral signing DTOs,
capability checks, BIP-110 size metadata validation, and verifier/finality
contracts. It does **not** contain a production `UniversalChainSigner`, PSBT
processor, transaction builder, Taproot interpreter, script engine, RPC client,
or broadcast path.

The concrete signer remains downstream. The `BitcoinAdapter` in
[`src/adapters/mod.rs`](../../src/adapters/mod.rs) is an adapter for address,
fee, and state-root interfaces, not a transaction signer. The guide therefore
documents a handoff, not an in-Core end-to-end implementation.

Wallet/Gateway/downstream transaction adapters own Bitcoin transaction
construction, BIP-341 sighash and preimage selection, control-block and annex
handling, and BIP-342 script parsing and execution. The enclave SDK signer
signs only the exact supplied `SigningPayload::Message` bytes or
`SigningPayload::Digest` bytes in a `SignRequest`; Core does not construct or
interpret Bitcoin transactions or scripts.

## Canonical target and UCS boundary

For Bitcoin, the canonical target is
`SigningTarget::for_chain(Chain::Bitcoin)`, whose family is
`ChainFamily::BitcoinUtxo`. A concrete signer advertises the exact subset it
supports with `ChainSigningCapability` inside `SignerCapabilities`, normally
including `EcdsaSecp256k1` and/or `SchnorrSecp256k1`, plus only the operations
and address formats it actually implements.

The UCS boundary is:

1. The caller constructs a `SignRequest` with an explicit target, algorithm,
   `SigningPayload::Message` or `SigningPayload::Digest`, and secret-free
   `DerivationContext`.
2. `UniversalChainSigner::sign` checks target consistency, payload length,
   derivation metadata, and advertised capabilities before invoking the
   implementation hook.
3. The signer returns a `SignResponse` containing the signature, public
   verification key, address, and derivation context. Private keys, seeds,
   shares, and enclave handles never cross this boundary.

No Bitcoin-domain hashing, BIP-341/BIP-342 serialization, PSBT parsing, or
transaction construction is implicit in `SigningPayload`. The caller must pass
the exact message bytes or precomputed digest that the concrete signer is
contracted to sign.

### BIP-341/BIP-342 handoff

Wallet/Gateway/downstream transaction adapters own Taproot key-path and
script-path transaction construction, BIP-341 sighash/preimage selection,
control-block construction, annex handling, and BIP-342 Tapscript parsing and
execution, including `OP_SUCCESSx`/`OP_IF`/`OP_NOTIF` semantics. The enclave SDK
signer signs only the exact supplied `SigningPayload::Message` bytes or
`SigningPayload::Digest` bytes; it does not choose a sighash, construct a
preimage, or interpret a script. Core can carry the resulting bytes through a
`SignRequest` or validate public response metadata, but it does not construct
or interpret Bitcoin transactions or BIP-341/BIP-342 scripts. See the BIP-110
matrix for the current parser/interpreter boundary in
[`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md).

### BIP-110 preflight boundary

Before a Bitcoin transaction is signed or broadcast, a transaction-aware
downstream component must parse and classify the relevant surfaces and populate
`Bip110TransactionShape`. The enabled Core size contract is:

- applicable pushdata payload: **256 bytes maximum**;
- complete OP_RETURN ScriptPubKey: **83 bytes maximum**;
- complete non-OP_RETURN ScriptPubKey: **34 bytes maximum**; and
- applicable witness element: **256 bytes maximum**.

`Bip110Compliance::new()` enables those checks. The derived
`Bip110Compliance::default()` is intentionally **disabled**, so a caller that
uses `default()` must not treat a successful result as BIP-110 compliance. The
adapter still owns classification, BIP-16/BIP-141/BIP-341/BIP-342 context, and
deployment or grandfathering decisions. See
[`src/control_model/bip110.rs`](../../src/control_model/bip110.rs),
[`src/control_model/trust.rs`](../../src/control_model/trust.rs), and
[`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md).

## Participant ownership

| Participant | Owns in this flow | Does not own |
| --- | --- | --- |
| Core | `SignRequest`/`SignResponse`, `SignerCapabilities`, target validation, BIP-110 size contract, and protocol-verifier types | Bitcoin transaction construction, BIP-341 sighash/preimage selection, control-block/annex handling, BIP-342 script parsing/execution, private keys, network I/O, retries, or broadcast |
| `conxius-enclave-sdk` | Hardware-backed custody, derivation, concrete ECDSA/Schnorr signing, attestation, and the concrete UCS implementation | Gateway routing and persistence |
| Gateway | Intent orchestration, transaction-construction orchestration and adapter selection, provider selection, BIP-110 preflight policy, persistence, retries, broadcast, and external side effects | Key custody and replacement of Core DTO validation |
| Nexus | Bitcoin headers, UTXO/transaction observation, proof acquisition, and verifier backends | Signing keys and user approval |
| Wallet | User review/approval, fee and destination policy, transaction-surface selection, and presentation of signer/verifier results | Canonical protocol DTO definitions, transaction/script construction truth, or enclave internals |

## Sequence and ownership

| Step | Owner | Inputs and evidence | Core contract / boundary | Output | Stop condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Wallet + Gateway | Destination, amount, fee policy, derivation purpose, transaction or message bytes | Select `Chain::Bitcoin`; validate the intended operation and target family | A reviewable unsigned payload and policy context | Missing destination, amount, or policy data |
| 2 | Wallet/Gateway + downstream transaction adapter | UTXOs, scripts, BIP-341 sighash/preimage context, control block, annex, BIP-342 script context, or message/digest bytes | Core does not build or interpret the transaction/script; it accepts only explicit `SigningPayload` and secret-free derivation metadata | Canonical bytes ready for signing and, when applicable, a classified `Bip110TransactionShape` | Unsupported transaction surface or unclassified BIP-341/BIP-342 context |
| 3 | Gateway | Parsed script/witness sizes and transaction context | Call `Bip110Compliance::new()`/`Bip110TransactionShape::validate`; do not use disabled `Bip110Compliance::default()` as approval | Compliant size result or typed violations | Any applicable 256/83/34/256 limit violation or unsupported exception |
| 4 | Wallet + SDK signer | `SignRequest`, capability advertisement, user approval, attestation/policy context | `UniversalChainSigner::sign` fail-closed checks target, algorithm, payload, derivation, and response metadata | `SignResponse` with public verification metadata | Unsupported chain/algorithm/operation, malformed request, backend failure, or invalid response |
| 5 | Gateway + Nexus | Transaction id, Bitcoin proof/finality evidence, expected policy | `ProtocolVerifier` validates the request/result envelope and finality policy around a Nexus backend | Verified evidence or a non-final/invalid result | Evidence mismatch, stale/expired/malformed proof, policy block, or required finality not met |
| 6 | Gateway | Signed transaction and verified preconditions | Core has no broadcast API; Gateway owns the side effect and Nexus observes the result | Broadcast reference and later finality status | Never broadcast when preflight, signing, or required verification failed |

## Required inputs

- An explicit `SigningTarget` with `Chain::Bitcoin` and
  `ChainFamily::BitcoinUtxo`.
- A supported algorithm and operation advertised by `SignerCapabilities`.
- Non-empty message bytes or a digest with the exact declared digest length;
  Core performs no implicit hashing or domain separation.
- Secret-free derivation path and purpose metadata.
- For transaction signing, fully constructed bytes and the transaction-aware
  context needed by the downstream adapter to select BIP-341/BIP-342 rules and
  classify BIP-110 surfaces.
- For a state-proof flow, the requested chain identity, proof format, block
  reference, proof data, and any requested state root that the result must bind.
- For a transaction-finality flow, the requested chain identity, transaction
  identity, confirmation/finality policy, evidence, provenance, and downstream
  policy/trust context supplied by Nexus/Gateway.

## Required outputs

- A validated `SignResponse` containing a signature, public verification key,
  chain address, and matching derivation context.
- A BIP-110 compliance result with all applicable size vectors represented;
  `compliant` means only that the supplied size metadata passed the configured
  limits.
- For state-proof verification, a `ProofVerificationResult` with the requested
  chain, proof format, and block reference bound to the verified block; when a
  state root was requested, the result and verified block must bind to that
  requested root.
- For transaction finality, a `TransactionFinalityResult` with transaction
  identity, status, required and observed confirmations, policy/trust/finality/
  verification metadata, and provenance. Its `latest_block` is optional; a
  state root is not universally required for transaction finality.
- A Gateway-owned broadcast/observation record. Core does not create or persist
  it.

## Verification and finality boundary

The consumer must use the `ProtocolVerifier` façade, not call a backend hook
directly. A Nexus `ProtocolVerifierBackend` acquires Bitcoin evidence; the
façade validates capabilities, request/result identity, evidence binding,
provenance, trust policy, and finality postconditions. `TrustTier::Strict`
requires a light-client verification class. `NonFinalState` means the result is
not final when finality was required; it is not permission to broadcast or
settle.

The two verifier result shapes have different obligations:

- **State-proof flow:** `ProofVerificationResult` must bind the requested chain,
  proof format, and block hash/height reference to the verified block. If the
  request includes a state root, the result and verified block must bind to that
  requested root; the binding is conditional on the request.
- **Transaction-finality flow:** `TransactionFinalityResult` must carry the
  transaction identity, lifecycle status, required/observed confirmations,
  finality class, verification class, trust tier, verification status, and
  provenance. `latest_block` is an optional latest verified block reference, and
  no state root is universally required by this flow.

The `BitcoinAdapter::verify_state_proof` implementation currently returns
`Ok(true)` and its state root is static. That adapter behavior is not production
Bitcoin proof verification. A production flow must use a real Nexus verifier
backend and retain the façade's postcondition checks.

Message verification has a separate BIP-322 caveat: `Bip322Bridge` in
[`src/bitcoin/bip322.rs`](../../src/bitcoin/bip322.rs) is structural-only in
the current implementation. It can accept a `bc1`-prefixed address before full
validation and ultimately checks that a witness is non-empty; it does not
execute the constructed `to_spend`/`to_sign` script validation. Do not use it as
proof of message-signature authenticity.

## Retry versus terminal semantics

The following is a flow-level distinction, not an automatic retry mechanism:

- **Potentially retryable or waitable:** a provider/backend is temporarily
  unavailable; evidence has not arrived; a transaction is still pending and
  policy permits waiting; or a finality request returns a non-final status that
  can be observed again.
- **Terminal:** target/family mismatch, unsupported capability, malformed
  payload or derivation, invalid signer response, BIP-110 violation,
  unsupported/unclassified Taproot or Tapscript context, malformed or invalid
  proof, evidence-binding mismatch, expired evidence, verifier identity
  mismatch, policy block, or a required finality failure that the operation
  cannot wait through.

Operational retry classification is downstream policy owned by Gateway. Core
does not schedule retries, hide terminal errors, or convert a negative
verification result into success.

## Fail-closed boundaries

- Reject an inconsistent `SigningTarget` or an unadvertised capability before
  invoking the signer.
- Reject empty messages, invalid digest lengths, invalid public metadata, and
  mismatched response derivation.
- Reject any BIP-110 violation from the enabled validator and reject unknown
  transaction context instead of treating it as compliant.
- Require a real, policy-compatible `ProtocolVerifier` result before a
  finality-dependent action; never treat a static root or `Ok(true)` adapter
  stub as proof.
- Keep BIP-322 structural checks out of an authenticity decision.
- Do not broadcast or persist a successful-looking flow after any failed
  precondition, verification, or finality check.

## Known gaps and unsupported behavior

- This repository has no production `UniversalChainSigner` implementation;
  tests use deterministic mocks.
- There is no PSBT or transaction DTO contract, Taproot/Tapscript interpreter,
  Miniscript compiler, fee/broadcast client, or transaction builder in Core.
- BIP-341/BIP-342 handoff is documented, not implemented here.
- `Bip110Compliance::default()` is disabled; use `Bip110Compliance::new()` or
  an explicitly enabled configuration when a caller intends to enforce the
  size contract.
- BIP-110 activation, expiry, UTXO grandfathering, and script-context
  exceptions remain downstream parser/deployment concerns.
- BIP-322 verification is structural-only as described above.

## Source references

- UCS contract: [`src/signing.rs`](../../src/signing.rs) and
  [`docs/SIGNING_ARCHITECTURE.md`](../SIGNING_ARCHITECTURE.md)
- Protocol verification: [`src/verifier.rs`](../../src/verifier.rs) and
  [`docs/architecture/PROTOCOL_VERIFIER.md`](../architecture/PROTOCOL_VERIFIER.md)
- BIP-110 limits and caveats: [`src/control_model/bip110.rs`](../../src/control_model/bip110.rs),
  [`src/control_model/trust.rs`](../../src/control_model/trust.rs), and
  [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md)
- Bitcoin adapter and BIP-322: [`src/adapters/mod.rs`](../../src/adapters/mod.rs)
  and [`src/bitcoin/bip322.rs`](../../src/bitcoin/bip322.rs)
- Ownership boundaries: [`docs/ARCHITECTURE_BOUNDARIES.md`](../ARCHITECTURE_BOUNDARIES.md)
- Downstream contracts: [enclave SDK issue #179](https://github.com/Conxian/conxius-enclave-sdk/issues/179),
  [Gateway issue #245](https://github.com/Conxian/conxian-gateway/issues/245),
  [Nexus issue #163](https://github.com/Conxian/conxian-nexus/issues/163), and
  [Wallet issue #381](https://github.com/Conxian/conxius-wallet/issues/381)
