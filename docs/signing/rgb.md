# RGB signing and validation flow

## Status / support level

**Transition/seal interfaces with explicit rollout modes; no concrete signer.**
Core exposes RGB transition, single-use seal, contract lookup, and runtime mode
contracts. The current adapters contain permissive placeholder behavior, and
the rollout mode is not evidence of protocol completeness.

Core never holds private keys, accesses hardware, performs RPC or other
network I/O, persists state, or owns runtime retries. RGB has no dedicated
`Chain` enum variant; Bitcoin anchors must use an explicitly capability-gated
Bitcoin-family target, normally `Chain::Bitcoin`.

## End-to-end flow boundary

1. A downstream RGB client creates or receives a transition, seal commitment,
   contract ID, and any Bitcoin anchor transaction data.
2. [`RGBRuntime`](../../src/rgb/mod.rs) invokes an `RGBAdapter` according to
   `RGBExecutionMode`: `Disabled`, `Shadow`, or `Active`.
3. For Bitcoin anchor signing, the Wallet/Gateway supplies a concrete UCS
   request with the Bitcoin target, explicit payload, and derivation context.
   Core's signing contract validates the request; it does not create the RGB
   transition, construct the anchor transaction, or sign it itself.
4. Gateway owns node/provider calls, persistence, orchestration, and external
   side effects. Nexus observes Bitcoin anchors and supplies evidence through
   its verifier backend. The Enclave SDK / Wallet owns key custody and concrete
   signing.
5. RGB policy must distinguish validation results from observation. In
   particular, Shadow mode must never authorize enforcement or submission.

## Required inputs and outputs

| Boundary | Current Core representation |
| --- | --- |
| Anchor signing target | `SigningTarget { chain: Chain::Bitcoin, family: ChainFamily::BitcoinUtxo }`, subject to a concrete capability |
| Signing request/response | `SignRequest` / `SignResponse`; derivation metadata is public path/purpose metadata only |
| Transition validation | `validate_transition(transition_hex) -> Result<bool, RGBError>` |
| Seal validation | `verify_seal(utxo_txid, seal_commitment) -> Result<bool, RGBError>` |
| Contract lookup | `get_contract_details(contract_id) -> Result<String, RGBError>` |
| Runtime mode | `RGBExecutionMode::{Disabled, Shadow, Active}` |
| Disabled behavior | Returns `RGBError::GatedByRolloutMode` without invoking the adapter |
| Shadow behavior | Invokes transition/seal validation but discards the result and returns `Ok(true)`; this is observation-only/permissive and not enforcement |
| Active behavior | Delegates to the adapter; current stock/skeleton implementations still return permissive success for non-empty transition/seal inputs |
| Contract ID | `validate_contract_id(id_hex) -> Result<ContractId, RGBError>` |

## Ownership

| Owner | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | RGB adapter/runtime interfaces, rollout-mode gates, contract-ID representation, UCS request validation, and protocol error vocabulary | RGB node I/O, durable state, private keys, anchor construction, AluVM execution, or retries |
| Conxius Enclave SDK / Wallet | Hardware-backed custody, derivation, user approval, and signing of supported Bitcoin anchor transactions | RGB state machine orchestration, node lookups, persistence, or contract validation policy |
| Gateway (`conxian-gateway`) | RGB workflow coordination, node/provider integration, persistence, retry/reconciliation, rollout configuration, and network side effects | Private-key custody and treating Shadow results as authorization |
| Nexus (`conxian-nexus`) | Bitcoin anchor observation, proof acquisition, and verifier backends for evidence used by policy | RGB signing, node-backed application workflow, or runtime retry ownership |

## Retryable versus terminal failures

**Potentially retryable or reconcilable downstream failures:**

- temporary node/provider unavailability or contract lookup timeout;
- persistence or transport failure when no external state transition occurred;
- ambiguous anchor submission requiring Gateway/Nexus reconciliation;
- signer backend failure only when the concrete signer confirms no side effect.

**Terminal for the supplied input or policy:**

- invalid contract ID, empty transition, schema mismatch, transition rejection,
  seal rejection, or invalid UCS request;
- `GatedByRolloutMode` while the runtime is Disabled;
- a failed Active-mode validation;
- unavailable or unverifiable evidence when the caller requires enforcement.

The current `RGBError::PersistenceError` is an error vocabulary entry; Core does
not provide a persistence implementation or a retry classification for it.

## Fail-closed boundaries

- Disabled mode must block operations rather than silently pass them.
- Shadow mode is explicitly non-enforcing: its `Ok(true)` result must not
  authorize signing, broadcast, minting, transfer, or settlement.
- Active mode is not a guarantee that the underlying adapter performs complete
  schema, AluVM, or cryptographic seal verification. Downstream policy must
  require implementation-specific evidence.
- A Bitcoin-family anchor signer must still pass `SignerCapabilities::require`;
  RGB terminology does not grant a signer capability.
- Contract lookup or observation success does not establish transition validity
  or ownership of a seal.

## Current gaps / unsupported behavior

- The current stock and skeleton adapters do not implement complete RGB schema
  and AluVM transition validation or cryptographic single-use-seal checks.
- No RGB node client, durable stock/persistence layer, consignment workflow,
  anchor transaction builder, or concrete signer exists in Core.
- Shadow mode deliberately bypasses enforcement and is suitable only for
  observation/rollout exercises.
- Core does not own RGB runtime retries, network calls, or final settlement.

## Source links

- [Universal signing architecture](../SIGNING_ARCHITECTURE.md)
- [UCS contract and types](../../src/signing.rs)
- [RGB adapter and runtime modes](../../src/rgb/mod.rs)
- [BIP-110 RGB anchor surface](../BIP110_ALIGNMENT.md)
- [Protocol verifier ownership](../architecture/PROTOCOL_VERIFIER.md)
- [Core/Gateway boundary](../ARCHITECTURE_BOUNDARIES.md)
