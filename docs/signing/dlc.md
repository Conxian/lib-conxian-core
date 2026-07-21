# DLC signing flow

## Scope and current support status

This guide covers the Discreet Log Contract flow from intent through funding,
refund/CET signing, oracle attestation, and Bitcoin settlement. Core currently
defines `DlcIntent`, `DlcStatus`, construction-only intent creation, and two
oracle/execution helpers. It does not define funding, refund, CET,
adaptor-signature, or transaction DTOs; it also does not build, persist,
broadcast, or monitor a DLC.

`DlcManager::try_verify_oracle_attestation` performs a real secp256k1
point-equation check for caller-supplied oracle public key, nonce point, outcome
message, and signature scalar; the boolean `verify_oracle_attestation` wrapper
preserves compatibility by returning `false` on any failure.
`DlcManager::verify_execution` is a compatibility-only typed unsupported
boundary because its shallow arguments cannot bind the nonce, outcome message,
oracle key, or expiry. The new `verify_execution_attestation` helper binds the
outcome hash, collateral, expiry, and oracle equation, but it is not a CET or
Bitcoin finality verifier.

`DlcManager::create_intent` is construction-only: it copies the supplied oracle
key bytes, collateral, outcome hash, and expiry into a `DlcIntent`. It does not
validate oracle-key encoding, reject zero collateral, check expiry semantics, or
bind the intent to a funding contract or counterparty agreement. Those checks
belong to the caller/Gateway policy before the intent is accepted or used.

## Canonical target and UCS boundary

DLC is Bitcoin-native and has no separate `Chain::DLC` variant. The canonical
target for funding, refund, and CET transaction bytes is
`SigningTarget::for_chain(Chain::Bitcoin)` with `ChainFamily::BitcoinUtxo`.
Oracle messages are protocol inputs to the attestation verifier; they are not a
new chain target. A concrete signer advertises the exact ECDSA/Schnorr
algorithm, operation, and address format it can safely implement.

The UCS boundary is intentionally narrow:

1. Gateway/Wallet/downstream DLC code constructs the exact transaction bytes or
   digest and a secret-free `SignRequest`.
2. `SignerCapabilities` and `UniversalChainSigner::sign` validate target,
   capability, payload, and
   derivation metadata before calling the SDK implementation.
3. The SDK returns a public `SignResponse`; Core does not receive the adaptor
   secret, oracle private key, funding key, or CET key material.

Core's `DlcIntent` and oracle helper do not imply that the requested bytes are a
valid DLC transaction or that an oracle outcome is bound to the intent. The
caller must perform those bindings explicitly.

## Participant ownership

| Participant | Owns in this flow | Does not own |
| --- | --- | --- |
| Core | `DlcIntent`/`DlcStatus` models, construction-only intent creation, UCS request/response validation, oracle-attestation primitive, and verifier/finality contracts | Oracle-key encoding, zero-collateral/expiry/contract-binding policy; funding/refund/CET construction, adaptor-signature protocol, oracle key custody, persistence, or broadcast |
| `conxius-enclave-sdk` | Hardware-backed signer custody and concrete Bitcoin signing/adaptor-signature implementation where supported | Oracle observation and Gateway workflow state |
| Gateway | Intent construction and policy validation, offer/accept/sign/execute/refund orchestration, persistence, counterparty/provider routing, retries, and broadcast effects | Private-key custody or replacing Core's attestation contract |
| Nexus | Oracle/Bitcoin observation, transaction proofs, block/finality evidence, and verifier backends | DLC key custody, transaction construction, or user approval |
| Wallet | User review/approval, collateral/expiry/outcome display, and fee policy | Oracle truth, CET construction, or proof verification |

## Sequence and ownership

| Step | Owner | Inputs and evidence | Core contract / boundary | Output | Stop condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Gateway | Oracle public key, collateral, outcome hash, expiry block, funding/counterparty binding | `DlcManager::create_intent` is construction-only and copies the supplied fields; caller/Gateway validates oracle-key encoding, non-zero collateral, expiry semantics, and contract binding | Reviewable, Gateway-owned intent | Invalid oracle-key encoding, zero collateral, invalid expiry, or missing contract/counterparty binding |
| 2 | Gateway + counterparty | Contract descriptors, collateral UTXOs, payout policy, refund timelock, oracle event mapping | Core has no funding/refund DTO or builder; downstream code owns exact bytes | Unsigned funding/refund transaction payloads | Missing contract binding, unsupported script, or incomplete counterparty agreement |
| 3 | Wallet + SDK | `SignRequest`, Bitcoin capability, policy/attestation context | UCS validates request/response metadata | Public `SignResponse` for each transaction | Unsupported capability, malformed payload, backend failure, or invalid response |
| 4 | Oracle | Nonce point `R`, outcome message, signature scalar `s`, oracle public key | `try_verify_oracle_attestation` checks the point equation; `verify_execution_attestation` also binds the message to `DlcIntent::outcome_hash` and checks expiry/collateral | Cryptographically valid/invalid attestation result | Invalid key/point/scalar, equation failure, wrong outcome binding, or expired event |
| 5 | Downstream | Verified oracle outcome, funding outpoint, adaptor data, payout outputs | Core supplies no CET/adaptor DTO; use a Bitcoin `SignRequest` only for exact constructed bytes | Signed CET or refund path | No verified attestation, missing funding state, unsupported adaptor flow, or policy block |
| 6 | Nexus + Gateway | Funding/CET/refund txid, Bitcoin proof and finality evidence | `ProtocolVerifier` validates transaction/state/finality result around a real backend | Verified settlement or non-final result | Invalid/stale proof, evidence mismatch, non-final required state, or rejected transaction |

## Required inputs

- Caller/Gateway-validated `DlcIntent` fields: oracle public key, collateral,
  outcome hash, and expiry block. Before accepting the model, caller/Gateway
  policy must validate oracle-key encoding, non-zero collateral, expiry
  semantics, and funding/contract/counterparty binding; `create_intent` does
  not perform those checks.
- Fully constructed funding, refund, or CET bytes/digest and the Bitcoin
  signing target/capability for each signing operation.
- For oracle verification: exact oracle public key bytes, nonce point bytes,
  outcome message bytes, and 32-byte signature scalar bytes.
- For a fully bound execution, the exact nonce point, outcome message,
  signature scalar, current block, and intent must be passed to
  `verify_execution_attestation`; the helper binds the message to the intent's
  `outcome_hash`.
- Bitcoin transaction proof/finality evidence and provenance supplied by Nexus
  when settlement or refund policy requires it.

## Required outputs

- A validated `SignResponse` for each actual funding/refund/CET transaction
  surface.
- A positive result from `verify_execution_attestation` before a CET can be
  considered executable; this still requires downstream CET/funding checks.
- A `ProtocolVerifier` finality result for funding, CET, or refund settlement
  when required by policy.
- A Gateway-owned `DlcStatus`/workflow record. Core does not create a broadcast
  receipt or declare `Executed` from `verify_execution` alone.

## Verification and finality boundary

`try_verify_oracle_attestation` is a Core cryptographic primitive for the
supplied attestation tuple. `verify_execution_attestation` adds the required
outcome-hash, collateral, and expiry bindings before calling that primitive.
The caller still owns outcome encoding, event-id/nonce policy, oracle-key-set
policy, funding, contract, and counterparty binding. `verify_execution` returns
typed unsupported and must never be used as a production authorization gate.

Nexus supplies Bitcoin transaction/block evidence through a
`ProtocolVerifierBackend`; Gateway consumes the `ProtocolVerifier` façade. The
façade validates capabilities, transaction and chain identity, proof/result
binding, provenance, trust/finality policy, and required finality. A finality
result with `NonFinalState` is not settlement authority. A verified oracle
attestation is also not Bitcoin transaction finality.

## Retry versus terminal semantics

- **Potentially retryable or waitable:** counterparty/provider unavailability,
  delayed oracle event before expiry, missing Bitcoin confirmations, or a
  pending funding/CET observation while the contract remains safely open.
- **Terminal:** malformed intent or transaction payload, unsupported signer
  capability, invalid signer response, failed oracle equation, outcome/hash
  mismatch, expired oracle event, missing funding binding, unsupported adaptor
  flow, malformed/stale proof, evidence mismatch, policy block, or required
  finality not reached before the refund/expiry boundary.

Operational retry classification is downstream policy owned by Gateway. Core
does not retry oracle calls, reconstruct CETs, or promote
`verify_execution`'s compatibility result into a cryptographic approval.

## Fail-closed boundaries

- Require an explicit Bitcoin target and capability for every funding, refund,
  and CET signing request; do not infer support from `DlcIntent` construction.
- Require caller/Gateway policy to validate oracle-key encoding, non-zero
  collateral, expiry semantics, and funding/contract/counterparty binding;
  `DlcManager::create_intent` is not a policy validator.
- Require real `verify_execution_attestation` success before CET authorization;
  it does not replace funding, CET, adaptor-signature, or finality checks.
- Do not use `verify_execution` as a substitute for oracle attestation, adaptor
  signature verification, or CET validation.
- Require a policy-compatible `ProtocolVerifier` result before a
  finality-dependent settlement or refund side effect.
- Reject missing or unsupported transaction construction context rather than
  signing an opaque or guessed CET/funding payload.
- Stop after any failed signer, oracle, contract-binding, proof, policy, or
  finality precondition.

## Known gaps and unsupported behavior

- No funding, refund, CET, adaptor-signature, oracle-event, or transaction DTOs
  exist in Core.
- No DLC transaction builder, contract execution engine, persistence, counterparty
  coordinator, oracle-set manager, or broadcast integration exists here.
- `verify_execution_attestation` does not verify funding, CET, adaptor
  signatures, or Bitcoin finality.
- `verify_execution` is compatibility-only and always returns typed
  `UnsupportedExecutionVerification`.
- No production DLC `UniversalChainSigner` or DLC-aware
  `ProtocolVerifierBackend` is implemented in this repository.

## Source references

- UCS contract: [`src/signing.rs`](../../src/signing.rs) and
  [`docs/SIGNING_ARCHITECTURE.md`](../SIGNING_ARCHITECTURE.md)
- DLC intent and verification helpers: [`src/protocol/dlc.rs`](../../src/protocol/dlc.rs)
- Bitcoin-family mapping: [`src/control_model/trust.rs`](../../src/control_model/trust.rs)
- Protocol verification: [`src/verifier.rs`](../../src/verifier.rs) and
  [`docs/architecture/PROTOCOL_VERIFIER.md`](../architecture/PROTOCOL_VERIFIER.md)
- Bitcoin/BIP-110 boundary: [`src/control_model/bip110.rs`](../../src/control_model/bip110.rs)
  and [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md)
- Architecture boundaries: [`docs/ARCHITECTURE_BOUNDARIES.md`](../ARCHITECTURE_BOUNDARIES.md)
- Downstream contracts: [enclave SDK issue #179](https://github.com/Conxian/conxius-enclave-sdk/issues/179),
  [Gateway issue #245](https://github.com/Conxian/conxian-gateway/issues/245),
  [Nexus issue #163](https://github.com/Conxian/conxian-nexus/issues/163), and
  [Wallet issue #381](https://github.com/Conxian/conxius-wallet/issues/381)
