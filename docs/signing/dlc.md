# DLC signing and oracle-attestation flow

## Status / support level

**Intent representation and oracle-attestation verification only.** Core can
create a [`DlcIntent`](../../src/protocol/dlc.rs) and perform the current
equation-based secp256k1 oracle-attestation check. It does not provide a DLC
signer or settlement workflow.

Core never holds private keys, accesses hardware, performs RPC or other
network I/O, persists state, or owns runtime retries. DLC has no dedicated
`Chain` enum variant; Bitcoin funding and settlement signing must use an
explicitly capability-gated Bitcoin-family target and keep DLC metadata outside
the generic signing payload unless a downstream protocol contract defines it.

## End-to-end flow boundary

1. Counterparties or Gateway create and exchange a [`DlcIntent`](../../src/protocol/dlc.rs)
   containing the oracle public key, collateral, outcome hash, and expiry
   block.
2. The downstream application acquires the oracle event and nonce/signature
   material. Core does not query an oracle or decide which event is canonical.
3. [`DlcManager::verify_oracle_attestation`](../../src/protocol/dlc.rs)
   verifies the supplied `(R, s)` attestation against the oracle public key and
   outcome message. A false result blocks the outcome.
4. The Enclave SDK / Wallet and Gateway own funding/refund transaction
   construction, adaptor-signature coordination, concrete signing, workflow
   persistence, and Bitcoin submission when those capabilities exist.
5. Nexus observes Bitcoin funding/settlement state and supplies independently
   verified evidence. Core's DLC intent and verifier do not create or execute a
   contract on chain.

## Required inputs and outputs

| Boundary | Current Core representation |
| --- | --- |
| Intent input | `DlcManager::create_intent(oracle_pubkey, collateral, outcome, expiry)` |
| Intent output | `DlcIntent { oracle_pubkey, collateral_sats, outcome_hash, expiry_block }` |
| Lifecycle vocabulary | `DlcStatus::{Offered, Accepted, Signed, Executed, Refunded}`; Core does not persist or drive these transitions |
| Attestation inputs | `oracle_pubkey`, `nonce_point` (`R`), `outcome_msg`, and `signature_scalar` (`s`) |
| Attestation output | `bool` from `verify_oracle_attestation`; `false` means the attestation must be rejected |
| Compatibility execution check | `verify_execution(intent, oracle_signature)` only checks a non-empty, non-zero, at-least-32-byte value and positive collateral; it is not full DLC execution verification |
| Bitcoin signing boundary | `SignRequest` / `SignResponse` with `Chain::Bitcoin` / `ChainFamily::BitcoinUtxo`, only if the concrete signer advertises the requested operation |
| Bitcoin preflight | Funding/refund/settlement transaction bytes must be classified by a downstream adapter before applying the [`Bip110TransactionShape`](../../src/control_model/bip110.rs) contract |

The oracle-attestation verifier performs cryptographic point/scalar parsing and
checks the relation `s*G = R + H(R, m)*P` for the supplied inputs. That verifies
the supplied attestation; it does not acquire the oracle event, validate the
full DLC contract, or produce a settlement transaction.

## Ownership

| Owner | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | DLC intent fields, status vocabulary, oracle-attestation verification, UCS validation, and Bitcoin preflight contracts | CET construction, funding/refund builders, adaptor-signature sessions, private keys, oracle acquisition, RPC, persistence, or retries |
| Conxius Enclave SDK / Wallet | Hardware-backed custody, key derivation, adaptor-signature policy, concrete signing, and user approval for supported Bitcoin flows | Oracle event acquisition, Gateway workflow state, or Core's intent/verifier definitions |
| Gateway (`conxian-gateway`) | Counterparty/oracle workflow coordination, persistence, provider selection, retries/reconciliation, transaction submission, and external side effects | Private-key custody and bypassing oracle or capability checks |
| Nexus (`conxian-nexus`) | Bitcoin observation, funding/settlement evidence, proof acquisition, and verifier backends | Oracle event acquisition by assumption, DLC signing, CET construction, or runtime retry policy |

## Retryable versus terminal failures

**Potentially retryable or reconcilable downstream failures:**

- temporary oracle/provider/node unavailability before an event is acquired;
- network or observation timeout while checking funding or settlement;
- counterparty/signing coordination failure when no signature or transaction
  side effect has been confirmed;
- ambiguous submission requiring Gateway/Nexus reconciliation.

**Terminal for the supplied contract or evidence once the relevant validator
establishes the condition:**

- malformed public key, nonce point, scalar, or invalid attestation (`false`)
  from Core's `verify_oracle_attestation`;
- outcome mismatch, expired block, invalid collateral, or inconsistent intent
  only after a downstream DLC contract, builder, or orchestrator validates the
  condition;
- failed capability, payload, address, or signer-response validation;
- any CET/funding/refund construction failure that the downstream builder
  classifies as invalid rather than transient.

The current Core helpers do not perform the downstream checks in the second
bullet. `verify_oracle_attestation` checks the supplied `(R, s)` equation and
input encodings for the supplied outcome message; it does not compare that
message with `DlcIntent::outcome_hash` or validate expiry, full collateral, or
intent consistency. `verify_execution` only checks a non-empty, non-zero,
at-least-32-byte signature and `collateral_sats > 0`; it does not perform those
contract checks or authorize settlement.

## Fail-closed boundaries

- Oracle event acquisition is required before attestation verification can be
  meaningfully applied; Core does not invent an event or outcome.
- A false cryptographic attestation result blocks execution.
- A valid oracle attestation does not prove that the intent, collateral,
  funding transaction, refund path, or settlement transaction is correct.
- The compatibility `verify_execution` check is insufficient for authorization
  because it does not verify a DLC signature or contract execution.
- No CET builder, funding/refund builder, adaptor-signature coordinator, or
  signer capability exists in Core; those missing capabilities must fail closed
  rather than fall back to a generic signing path.
- Bitcoin transaction policy, including BIP-110 classification, must be
  satisfied before a downstream signer or broadcaster proceeds.

## Current gaps / unsupported behavior

- No CET construction, funding/refund transaction builder, DLC script/policy
  compiler, or adaptor-signature coordination exists in Core.
- No oracle event acquisition, nonce management, persistence, counterparty
  protocol, or Bitcoin network submission exists here.
- `verify_execution` is retained as a compatibility helper and is not a
  complete DLC execution verifier.
- No concrete DLC `UniversalChainSigner` or chain-specific capability is
  advertised by this module.

## Source links

- [Universal signing architecture](../SIGNING_ARCHITECTURE.md)
- [UCS contract and types](../../src/signing.rs)
- [DLC intent and oracle verifier](../../src/protocol/dlc.rs)
- [BIP-110 Bitcoin transaction handoff](../BIP110_ALIGNMENT.md)
- [Protocol verifier ownership](../architecture/PROTOCOL_VERIFIER.md)
- [Core/Gateway boundary](../ARCHITECTURE_BOUNDARIES.md)
