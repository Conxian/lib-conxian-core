# Stacks and sBTC signing flow

## Status / support level

**Pilot intent/status model only.** Core exposes sBTC peg-in and peg-out
intent shapes and a small Stacks-family adapter surface. The current behavior
contains placeholder address/timestamp values and an unconditional finalized
status path. There is no concrete Stacks signer, Clarity executor, or complete
sBTC bridge workflow in Core.

Core never holds private keys, accesses hardware, performs RPC or other
network I/O, persists state, or owns runtime retries. A `Chain::Stacks` entry
maps to `ChainFamily::BitcoinUtxo`, but that taxonomy entry does not imply a
signer or a working peg flow.

## End-to-end flow boundary

1. The Wallet or Gateway collects a peg-in or peg-out intent and validates the
   user-facing amount and destination in the application layer.
2. [`SBTCBridge::initiate_peg_in`](../../src/stacks/mod.rs) or
   `initiate_peg_out` creates an [`SBTCIntent`](../../src/stacks/mod.rs).
   These methods represent coordination intent; they do not submit Bitcoin,
   mint/burn sBTC, or execute Clarity.
3. For a concrete Bitcoin L1 signing operation, a downstream signer uses the UCS
   target `Chain::Bitcoin`; for a Stacks transaction signing operation, it uses
   `Chain::Stacks`. Both targets map to the coarse `ChainFamily::BitcoinUtxo`,
   but the exact `SigningTarget` remains significant for capability matching,
   including the requested algorithm, operation, and `AddressFormat::StacksC32`
   where appropriate.
4. The Enclave SDK / Wallet signs constructed transactions. Gateway coordinates
   peg state, persistence, provider calls, and retries. Nexus observes Bitcoin
   and Stacks evidence and supplies finality information.
5. Core can validate the represented request or adapter result, but it cannot
   turn a placeholder state into proof of a completed peg.

## Required inputs and outputs

| Boundary | Current Core representation |
| --- | --- |
| Signing target | `SigningTarget { chain: Chain::Stacks, family: ChainFamily::BitcoinUtxo }` |
| Signing request | `SignRequest { target, algorithm, payload, derivation }`; use `SigningPayload::Message` or explicit `Digest` |
| Address format | `AddressFormat::StacksC32` is available to a concrete capability; Core does not perform full Stacks checksum/network validation here |
| Peg-in input | `amount_sats: u64`, `btc_txid: &str` |
| Peg-in output | `SBTCIntent { intent_id, amount_sats, stacks_address, bitcoin_txid: Some(..), state: BitcoinConfirmed, created_at_epoch }` |
| Peg-out input | `amount_sats: u64`, `stacks_address: &str` |
| Peg-out output | `SBTCIntent { intent_id, amount_sats, stacks_address, bitcoin_txid: None, state: Pending, created_at_epoch }` |
| Status lookup | `get_status(intent_id: &str) -> Result<SBTCState, StacksError>`; current implementation ignores the ID and returns `Finalized` |
| Finality helper | `StacksNakamoto::verify_bitcoin_finality(stacks_block)` currently returns `true` for any non-zero block number; it is not proof acquisition or cryptographic finality verification |

The current peg-in response uses the placeholder address `ST123...` and the
fixed placeholder timestamp `1718363200`. The current peg-out response copies
the supplied Stacks address but still uses that placeholder timestamp.

## Ownership

| Owner | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | `SBTCIntent`/`SBTCState`, Stacks adapter contracts, UCS validation, and protocol-level field invariants | Private keys, Clarity execution, bridge custody, RPC, persistence, or retries |
| Conxius Enclave SDK / Wallet | User authorization, address derivation, hardware-backed signing, and signing of concrete Bitcoin/Stacks transactions | Peg orchestration, chain observation, or Core state persistence |
| Gateway (`conxian-gateway`) | Peg workflow orchestration, provider calls, persistence, idempotency, retry/reconciliation, and external side effects | Private-key custody and treating Core's placeholder status as settlement evidence |
| Nexus (`conxian-nexus`) | Bitcoin/Stacks observation, header/finality evidence, and proof verification backends | Signing, peg custody, Clarity execution, or application retry policy |

## Retryable versus terminal failures

**Potentially retryable or reconcilable outside Core:**

- `FinalityTimeout` while Gateway/Nexus checks chain state;
- transient node, provider, signer-coordination, or federation failures;
- an ambiguous submission where the workflow must query the intent and chain
  before repeating an operation.

`PegInFailed`, `PegOutFailed`, and `SignerCoordinationError` carry strings, not
retry classifications. Gateway must use provider-specific evidence and
idempotency rules before retrying.

**Terminal for the supplied request or evidence:**

- empty Bitcoin transaction ID (`InvalidTransaction`);
- empty or malformed destination (`InvalidAddress`);
- invalid UCS target, payload, derivation, capability, or signer response;
- a negative or malformed downstream proof;
- an explicitly failed peg after reconciliation confirms the failure.

## Fail-closed boundaries

- Do not treat `SBTCState::Finalized` from the current `get_status` method as
  proof of finality; the implementation returns it unconditionally.
- Do not treat `StacksNakamoto::verify_bitcoin_finality` as a light client or
  cryptographic proof. A non-zero block number is only a structural placeholder
  in the current adapter.
- Missing UCS capability for the exact Stacks target, algorithm, or operation
  must block signing.
- Placeholder addresses and timestamps must not be used for value-bearing
  routing, settlement, or audit evidence.
- Core validation success does not authorize a Clarity call, mint, burn, or
  Bitcoin broadcast.

## Current gaps / unsupported behavior

- No concrete Stacks `UniversalChainSigner` is implemented in Core.
- No real sBTC peg-in/peg-out state machine, bridge custody, mint/burn flow,
  transaction construction, or network submission exists here.
- The adapter does not validate full Stacks address semantics or obtain chain
  finality evidence.
- Clarity contract execution and Stacks-specific transaction encoding remain
  downstream responsibilities.

## Source links

- [Universal signing architecture](../SIGNING_ARCHITECTURE.md)
- [UCS contract and types](../../src/signing.rs)
- [Stacks/sBTC adapter](../../src/stacks/mod.rs)
- [Chain and family mapping](../../src/control_model/trust.rs)
- [Core/Gateway boundary](../ARCHITECTURE_BOUNDARIES.md)
- [Protocol verifier ownership](../architecture/PROTOCOL_VERIFIER.md)
