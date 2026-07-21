# Stacks / sBTC signing flow

## Scope and current support status

This guide covers the signing and coordination boundary for Stacks and the
sBTC peg-in/peg-out lifecycle. Core supplies the `StacksAdapter` interface,
`SBTCIntent`/`SBTCState` protocol models, a pilot `SBTCBridge`, and a small
`ContractBridge` helper for serialized Clarity calls. Gateway constructs,
applies policy to, persists, and advances each authoritative `SBTCIntent`;
Core does not own that workflow. These are protocol models and pilot helpers,
not a production sBTC signer, threshold coordinator, transaction serializer,
bridge relayer, or finality service.

The current pilot is intentionally qualified: `SBTCBridge::initiate_peg_in`
rejects only an empty Bitcoin txid and returns a placeholder Stacks address and
timestamp, while `SBTCBridge::initiate_peg_out` rejects only an empty Stacks
address. `SBTCBridge::get_status` returns `StacksError::StatusUnavailable` for a
non-empty intent id and never fabricates `SBTCState::Finalized` without an
evidence provider. These helper checks and outputs are not production
address/transaction validation or authoritative persisted state.
The pilot status path is not hardcoded to `SBTCState::Finalized`; it remains
unavailable without provider-backed evidence.

## Canonical target and UCS boundary

For a Stacks-side signing operation, the canonical target is
`SigningTarget::for_chain(Chain::Stacks)` with `ChainFamily::BitcoinUtxo` and a
capability such as `EcdsaSecp256k1` plus `AddressFormat::StacksC32` when the
concrete signer truly supports it. A Bitcoin-side peg transaction is a
separate request with `SigningTarget::for_chain(Chain::Bitcoin)`; the sBTC
intent state does not change the target of the bytes being signed.

The UCS boundary is the same explicit request/response contract used by every
chain:

1. Gateway or Wallet supplies a `SignRequest` with the exact Stacks or Bitcoin
   transaction/message bytes, algorithm, target, and secret-free derivation
   context.
2. `SignerCapabilities::require` and `UniversalChainSigner::sign` reject an
   unsupported chain, algorithm, operation, family, payload, or derivation
   before the SDK implementation is called.
3. The SDK returns a `SignResponse` with only public verification metadata.

`ContractBridge::create_signed_call` directly signs a JSON serialization with
an ECDSA key, but it is not wired into `UniversalChainSigner`. Its output must
not be treated as proof that a Stacks transaction, Clarity ABI, nonce, fee, or
network envelope is valid.

### Bitcoin-side BIP-110 boundary

BIP-110 applies only to serialized Bitcoin peg transaction surfaces: applicable
pushdata, output `ScriptPubKey` bytes, and applicable witness elements. It does
not apply to Stacks transactions, Clarity calls or serialization, or sBTC
ledger/intent/state data. Before a Bitcoin-side signing or broadcast decision,
Wallet/Gateway/downstream transaction adapters must parse the serialized
Bitcoin transaction, classify the constrained surfaces, and validate the
result with an enabled `Bip110Compliance` configuration. Core can carry the
resulting shape and compliance outcome, but it does not parse or classify the
transaction; an unclassified surface or disabled compliance configuration is
not approval. See [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md) and
[`src/control_model/bip110.rs`](../../src/control_model/bip110.rs).

## Participant ownership

| Participant | Owns in this flow | Does not own |
| --- | --- | --- |
| Core | `SignRequest`/`SignResponse`, Stacks target/family mapping, the `SBTCIntent`/`SBTCState` DTOs/models, and verifier/finality contracts | Constructing, policy-validating, or persisting sBTC intents; signer quorum, bridge custody, network calls, retries, or broadcast |
| `conxius-enclave-sdk` | Key custody, derivation, attested ECDSA signing, and concrete UCS implementation for any advertised Stacks/Bitcoin capability | Peg orchestration or lifecycle persistence |
| Gateway | Intent construction and policy validation, peg-in/out workflow, signer coordination, persistence, provider selection, retry policy, and external bridge effects | Private-key custody and replacement of Core state contracts |
| Nexus | Bitcoin headers/transactions, Stacks observations, proof acquisition, and verifier backends | User approval and signing keys |
| Wallet | User approval, destination/amount/fee review, and presentation of peg status | Bridge settlement, canonical state transitions, or SDK internals |

## Sequence and ownership

| Step | Owner | Inputs and evidence | Core contract / boundary | Output | Stop condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Gateway | Peg direction, amount, Bitcoin txid for peg-in or Stacks address for peg-out | Core supplies the `SBTCIntent`/`SBTCState` model; Gateway owns construction, policy validation, persistence, and lifecycle transitions | Gateway-owned intent record with an explicit state | Empty txid/address, invalid amount, or missing policy context |
| 2 | Wallet + Gateway | Bitcoin-side lock/release bytes or Stacks-side contract-call bytes | Select `Chain::Bitcoin` or `Chain::Stacks` per bytes; advertise exact UCS capability | Reviewable unsigned payload | Target/capability mismatch or unsupported transaction envelope |
| 3 | Gateway/downstream adapter | Serialized Bitcoin peg transaction and parsed pushdata, ScriptPubKey, and witness surfaces | Core validates supplied shape metadata through enabled BIP-110 compliance; it does not parse or classify the transaction | BIP-110 compliance result or typed violations | Unknown/unclassified surface, disabled compliance, or any applicable limit violation |
| 4 | SDK | `SignRequest`, derivation purpose, attestation/policy evidence | UCS validates request and response; Core does not serialize or broadcast Stacks transactions | `SignResponse` with public verification metadata | Unsupported operation, malformed payload, backend failure, or invalid response |
| 5 | Nexus | Bitcoin txid, headers, proof data, state reference | `ProtocolVerifier` validates proof/result identity and provenance; `TransactionFinalityRequest` can require Bitcoin finality | Verified or non-final Bitcoin evidence | Invalid/malformed/stale proof, evidence mismatch, policy block, or required finality not met |
| 6 | Gateway | Verified evidence, signer acknowledgements, bridge/provider response | Core carries the DTO/model; Gateway owns the transition workflow and persistence | Updated Gateway-owned intent record and operational status | Missing signer quorum, provider rejection, or lifecycle invariant failure |
| 7 | Nexus + Gateway | Peg-in/out transaction and observation/finality evidence | Core validates the supplied finality result; it does not infer settlement from an intent alone | `Finalized`, `Failed`, or still-pending downstream status | Never mark finalized from the pilot's hardcoded status or an unverified observation |

## Required inputs

- A direction-specific intent: Bitcoin txid and amount for peg-in, or Stacks
  address and amount for peg-out.
- The correct UCS target for the transaction surface (`Chain::Bitcoin` for
  Bitcoin bytes, `Chain::Stacks` for Stacks bytes), an advertised algorithm,
  operation, and address format.
- Exact serialized bytes or an explicit digest; Core does not add Stacks
  transaction encoding, Clarity serialization, nonce handling, or domain
  separation.
- For a Bitcoin-side peg, the serialized Bitcoin transaction plus downstream
  parsed/classified BIP-110 surface metadata and an enabled compliance
  configuration before signing or broadcast.
- Secret-free derivation context and the user/policy approval evidence owned by
  Wallet and Gateway.
- For settlement decisions, a Bitcoin transaction identifier and
  `ProtocolVerifier` evidence/finality request supplied by Nexus/Gateway.

## Required outputs

- A validated `SignResponse` for each transaction or message that actually
  requires a signature.
- A BIP-110 compliance result for each Bitcoin-side peg transaction, produced
  from downstream parsing/classification with an enabled compliance
  configuration.
- An `SBTCIntent` whose `SBTCState` is persisted and advanced by Gateway only
  when the corresponding evidence and signer/provider acknowledgements exist.
- A `ProtocolVerifier` result for Bitcoin finality when the peg policy requires
  it. A non-final result remains observable but is not settlement authority.
- A Gateway-owned bridge/provider result and Wallet-visible status. Core does
  not create a relayer receipt or claim that a peg has completed.

## Verification and finality boundary

For a Bitcoin-side peg-in/out transaction, Nexus supplies a
`ProtocolVerifierBackend` and Gateway consumes the `ProtocolVerifier` façade.
The façade checks capabilities, chain and transaction identity, proof/result
postconditions, provenance, trust policy, and finality. Use
`TransactionFinalityRequest` with `require_finality: true` where a settlement
policy requires Bitcoin finality; `NonFinalState` is an explicit not-final
outcome.

`StacksNakamoto::verify_bitcoin_finality_checked` returns typed malformed or
unsupported errors because a block number alone is not a production Bitcoin
light-client proof and is not a replacement for `ProtocolVerifier`. The
deprecated boolean wrapper returns `false`. Likewise, `SBTCState::Finalized` is
a Core state label, not independently verified evidence.

BIP-110 applies to serialized Bitcoin peg transaction surfaces, not to Stacks
transactions, Clarity data, or sBTC ledger state. Bitcoin-side signing and
broadcast require downstream parsing/classification and an enabled compliance
configuration; a Core model, txid, address, or status label is not a BIP-110
decision.

## Retry versus terminal semantics

- **Potentially retryable or waitable:** missing Bitcoin confirmations,
  temporarily unavailable proof/provider, delayed signer coordination, or an
  intent that remains in a non-terminal `SBTCState` while policy permits
  observation.
- **Terminal:** invalid target/family/capability, malformed payload or address,
  invalid signer response, contradictory or stale proof, evidence-binding
  mismatch, policy block, rejected bridge operation, invalid lifecycle
  transition, or a finality requirement that cannot be satisfied.

Operational retry classification is downstream policy owned by Gateway. Core
does not retry signer calls, poll sBTC providers, or reinterpret an unavailable
status result as `Finalized`.

## Fail-closed boundaries

- Keep Bitcoin-side and Stacks-side signing targets distinct; never infer one
  from an sBTC intent alone.
- Apply BIP-110 only to parsed/classified serialized Bitcoin peg transaction
  surfaces, using enabled compliance before signing or broadcast; never apply
  it to Stacks, Clarity, or sBTC ledger data.
- Require the exact signer capability before invoking the SDK and validate the
  response metadata before accepting it.
- Do not advance an intent to a settlement state solely because a txid or
  address is non-empty.
- Do not use `StacksNakamoto::verify_bitcoin_finality` or an unavailable bridge
  status as production finality.
- Require a policy-compatible `ProtocolVerifier` result before a
  finality-dependent mint, burn, release, or user-visible settlement claim.
- Do not broadcast or release funds after a failed proof, signer, quorum,
  policy, or lifecycle precondition.

## Known gaps and unsupported behavior

- `SBTCBridge` is a pilot interface with placeholder address/time behavior and
  an unavailable `get_status` result until a provider supplies evidence.
- Core has no production sBTC peg DTOs, signer quorum/threshold workflow,
  Stacks transaction builder, Clarity ABI/nonce validator, bridge RPC client,
  or broadcast path.
- `ContractBridge::create_signed_call` is a direct ECDSA helper, not a UCS
  implementation and not a complete Stacks transaction signer.
- No production Stacks `UniversalChainSigner` or `ProtocolVerifierBackend` is
  implemented in this repository.

## Source references

- UCS contract: [`src/signing.rs`](../../src/signing.rs) and
  [`docs/SIGNING_ARCHITECTURE.md`](../SIGNING_ARCHITECTURE.md)
- Stacks and sBTC models: [`src/stacks/mod.rs`](../../src/stacks/mod.rs)
- Clarity signing helper: [`src/contract_bridge.rs`](../../src/contract_bridge.rs)
- Protocol verification: [`src/verifier.rs`](../../src/verifier.rs) and
  [`docs/architecture/PROTOCOL_VERIFIER.md`](../architecture/PROTOCOL_VERIFIER.md)
- Chain-family mapping: [`src/control_model/trust.rs`](../../src/control_model/trust.rs)
- Architecture boundaries: [`docs/ARCHITECTURE_BOUNDARIES.md`](../ARCHITECTURE_BOUNDARIES.md)
- Downstream contracts: [enclave SDK issue #179](https://github.com/Conxian/conxius-enclave-sdk/issues/179),
  [Gateway issue #245](https://github.com/Conxian/conxian-gateway/issues/245),
  [Nexus issue #163](https://github.com/Conxian/conxian-nexus/issues/163), and
  [Wallet issue #381](https://github.com/Conxian/conxius-wallet/issues/381)
