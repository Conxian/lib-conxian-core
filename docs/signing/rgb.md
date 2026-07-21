# RGB signing flow

## Scope and current support status

This guide covers RGB client-side validation, single-use seals, Bitcoin
anchoring, and the signing handoff for an RGB state transition or consignment.
Core exposes a custom `RGBAdapter` with `validate_transition`, `verify_seal`,
and contract lookup methods. It is not a `UniversalChainSigner`, a generic
chain adapter, an RGB node, a consignment store, or a production Bitcoin
transaction builder.

The v0.3.0 RGB boundary is intentionally fail-closed. `RGBStockAdapter` and
`RGBSkeletonAdapter` reject empty transition/seal inputs and return
`RGBError::VerificationUnavailable` for otherwise non-empty input.
`RGBRuntime::Shadow` maps a usable observation to the explicit
`RGBError::NonAuthoritativeShadow` outcome. Shadow mode is observational only
and **cannot authorize production signing or settlement**. It is
non-enforcing by design.

## Canonical target and UCS boundary

There is no separate `Chain::RGB` variant in the Core taxonomy. RGB is a
Bitcoin-adjacent protocol flow, so a Bitcoin anchor or Bitcoin transaction
surface uses `SigningTarget::for_chain(Chain::Bitcoin)` and
`ChainFamily::BitcoinUtxo`. An RGB transition or consignment is not silently
turned into a Bitcoin signing payload; the downstream SDK/client owns its
serialization and chooses the exact bytes or digest that require signing.

The UCS boundary is:

1. A Wallet/Gateway/client prepares the RGB operation and, if needed, the
   Bitcoin anchor transaction bytes.
2. Core validates the explicit Bitcoin `SignRequest` and the advertised
   `SignerCapabilities` entry;
   `SigningPayload` does not hash, domain-separate, or encode RGB data.
3. The concrete SDK signer returns a public `SignResponse`. RGB transition
   validation and seal checks remain separate protocol inputs/evidence.

The custom `RGBAdapter` methods are not a substitute for UCS capabilities or
for a production signer implementation.

## Participant ownership

| Participant | Owns in this flow | Does not own |
| --- | --- | --- |
| Core | RGB transition/seal interface, Bitcoin-family UCS DTO validation, rollout-mode semantics, and verifier/finality contracts | RGB node state, consignment persistence, private keys, anchor construction, or network I/O |
| `conxius-enclave-sdk` | Hardware-backed key custody and concrete signing for the Bitcoin anchor or other advertised surface | RGB node validation and Gateway workflow state |
| Gateway | Workflow orchestration, persistence, provider selection, retry policy, anchor broadcast, and external effects | RGB cryptographic truth or key custody |
| Nexus | Bitcoin anchor observation, proof/finality acquisition, and verifier backends; any RGB-aware observation it implements | Signing keys and user approval |
| Wallet | User approval, asset/recipient/fee policy, and display of transition/anchor status | Transition execution, proof verification, or SDK internals |

## Sequence and ownership

| Step | Owner | Inputs and evidence | Core contract / boundary | Output | Stop condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Wallet + Gateway | Contract id, transition/consignment, seal commitment, intended Bitcoin anchor | Select Bitcoin-family target only for the bytes that actually need signing | Reviewable RGB operation | Invalid contract id, empty transition/seal, or missing policy |
| 2 | SDK/client | Transition bytes, contract/schema context, anchor UTXO/txid, seal commitment | `RGBAdapter::validate_transition` and `verify_seal` are typed Core boundaries; existing adapters return unavailable without a real provider | Validation result and anchor inputs | Adapter rejection, disabled rollout mode, or unsupported proof context |
| 3 | Gateway + downstream Bitcoin adapter | UTXOs, commitment output/script/witness, sighash context | Core does not construct RGB consignments or Bitcoin transactions; BIP-110 applies only to parsed Bitcoin bytes | Unsigned anchor payload and classified size metadata | Unsupported anchor context or BIP-110 violation |
| 4 | Wallet + SDK | `SignRequest`, Bitcoin capability, attestation/policy evidence | UCS validates target, payload, derivation, capability, and response | `SignResponse` | Unsupported capability, malformed request, backend failure, or invalid response |
| 5 | Nexus | Anchor txid, Bitcoin proof/header/finality evidence | `ProtocolVerifier` validates evidence binding, provenance, policy, and finality around a real backend | Verified or non-final anchor evidence | Invalid/stale/malformed proof, mismatch, policy block, or non-final required state |
| 6 | Gateway | Validated transition, seal, signed anchor, verifier result | Core does not persist or authorize from Shadow mode; Gateway owns the side effect | Finalized operation record or pending status | Never authorize production from `NonAuthoritativeShadow` or unavailable verification |

## Required inputs

- RGB contract identifier, transition/consignment bytes, and schema/context
  owned by the RGB client or downstream adapter.
- Seal commitment and the Bitcoin UTXO/transaction reference it claims to
  anchor.
- A Bitcoin `SignRequest` for the actual anchor bytes or digest, with an
  advertised algorithm, operation, address format, and derivation context.
- For the anchor, parsed Bitcoin context needed for BIP-110 classification and
  an enabled compliance configuration when the policy requires it.
- Bitcoin proof/finality evidence and provenance supplied by Nexus/Gateway.

## Required outputs

- A transition/seal validation result whose enforcement mode is explicit.
- A validated Bitcoin `SignResponse` when an anchor signature is required.
- A `ProtocolVerifier` result for the Bitcoin anchor when finality or proof is a
  prerequisite.
- A Gateway-owned RGB/anchor record. Core does not produce a consignment
  receipt, broadcast result, or production authorization token.

## Verification and finality boundary

RGB client-side validation and seal verification are distinct from Bitcoin
anchor finality. Nexus owns Bitcoin observation and any real verifier backend;
Gateway consumes it through `ProtocolVerifier`. The façade checks the
capability advertisement, request/result identity, state-root/block binding,
provenance, trust tier, verification class, finality class, and verifier
identity. It does not replace RGB client-side validation or prove a seal merely
because a transition is non-empty.

`RGBRuntime::Active` propagates adapter results, `Disabled` returns
`GatedByRolloutMode`, and `Shadow` returns an explicit non-authoritative error
for a usable observation. Only an explicitly enforcing mode backed by real validation and policy may
authorize a production flow. A Bitcoin anchor can be final without proving an
RGB transition is valid, and a valid transition cannot by itself prove the
anchor is final.

## Retry versus terminal semantics

- **Potentially retryable or waitable:** delayed Bitcoin anchor confirmations,
  temporarily unavailable RGB node or Nexus provider, or a pending transition
  while the downstream protocol permits another observation attempt.
- **Terminal:** invalid contract id, empty/malformed transition or seal,
  disabled rollout mode, unsupported capability, invalid signer response,
  failed enforcing-mode validation, evidence mismatch, stale/expired proof,
  policy block, or required anchor finality not reached before an irreversible
  deadline.

Operational retry classification is downstream policy owned by Gateway. Core
does not retry node calls, turn Shadow mode into enforcement, or convert a
negative Active-mode result into success.

## Fail-closed boundaries

- Use `Chain::Bitcoin` for Bitcoin anchor signing; do not invent or imply a
  separate `Chain::RGB` target.
- Treat `Shadow` as non-enforcing. `NonAuthoritativeShadow` cannot authorize a
  production transition, seal, or release.
- Require real transition/seal validation in an enforcing mode and real
  Bitcoin proof/finality evidence where policy requires it.
- Apply BIP-110 only to parsed Bitcoin anchor surfaces, not to off-chain RGB
  consignments, seals, or state data.
- Reject any failed signer, transition, seal, proof, policy, or finality
  precondition before broadcasting or persisting a production result.

## Known gaps and unsupported behavior

- No separate `Chain::RGB` variant or RGB-specific UCS capability exists.
- Core has no RGB state-transition signer, consignment builder, seal cryptography,
  node/RPC integration, persistence, or Bitcoin anchor transaction DTO.
- `RGBStockAdapter` and `RGBSkeletonAdapter` return typed unavailable outcomes
  without full schema, AluVM, seal, or contract-state verification.
- `RGBRuntime::Shadow` is explicitly non-enforcing and cannot authorize
  production.
- No production RGB `UniversalChainSigner` or RGB-aware
  `ProtocolVerifierBackend` is implemented here.

## Source references

- UCS contract: [`src/signing.rs`](../../src/signing.rs) and
  [`docs/SIGNING_ARCHITECTURE.md`](../SIGNING_ARCHITECTURE.md)
- RGB adapter/runtime: [`src/rgb/mod.rs`](../../src/rgb/mod.rs)
- Chain-family mapping: [`src/control_model/trust.rs`](../../src/control_model/trust.rs)
- Protocol verification: [`src/verifier.rs`](../../src/verifier.rs) and
  [`docs/architecture/PROTOCOL_VERIFIER.md`](../architecture/PROTOCOL_VERIFIER.md)
- Bitcoin/BIP-110 boundary: [`src/control_model/bip110.rs`](../../src/control_model/bip110.rs)
  and [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md)
- Architecture boundaries: [`docs/ARCHITECTURE_BOUNDARIES.md`](../ARCHITECTURE_BOUNDARIES.md)
- Downstream contracts: [enclave SDK issue #179](https://github.com/Conxian/conxius-enclave-sdk/issues/179),
  [Gateway issue #245](https://github.com/Conxian/conxian-gateway/issues/245),
  [Nexus issue #163](https://github.com/Conxian/conxian-nexus/issues/163), and
  [Wallet issue #381](https://github.com/Conxian/conxius-wallet/issues/381)
