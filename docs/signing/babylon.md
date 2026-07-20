# Babylon Bitcoin staking signing flow

## Status / support level

**Bitcoin-family adapter primitives with structural proof checks only.** Core
represents a Babylon staking intent and exposes a generic adapter surface, but
it does not implement staking transaction construction, BTC-header acquisition,
cryptographic EOTS verification, or a concrete signer.

Core never holds private keys, accesses hardware, performs RPC or other
network I/O, persists state, or owns runtime retries. `Chain::Babylon` maps to
`ChainFamily::BitcoinUtxo`; the current [`BabylonAdapter`](../../src/babylon/mod.rs)
also reports `Chain::Bitcoin` from its `chain()` method. Neither fact is a
claim of Babylon signer support.

## End-to-end flow boundary

1. The Wallet or Gateway collects the staking parameters and obtains the
   staker/finality-provider public keys through the downstream custody and
   application layer.
2. Core represents the parameters as [`StakingIntent`](../../src/babylon/mod.rs)
   and exposes `UniversalChainAdapter` methods for address, fee, trust-tier,
   state-root, and proof-shape operations.
3. Nexus acquires Bitcoin headers, Babylon finality evidence, and EOTS
   material from the relevant observation sources. Gateway coordinates the
   workflow and persistence.
4. The Enclave SDK / Wallet constructs and signs the actual staking, delegation,
   unbonding, or withdrawal transaction if it has an explicit UCS capability.
5. Core's current proof hook can reject empty or malformed structural input;
   it must not be used as cryptographic proof of a Babylon finality decision.

## Required inputs and outputs

| Boundary | Current Core representation |
| --- | --- |
| Signing target | Prefer an explicit `SigningTarget` for `Chain::Babylon` / `BitcoinUtxo` when the downstream signer advertises it; the current adapter identity returns `Chain::Bitcoin` |
| Staking intent | `StakingIntent { staker_pubkey, finality_provider_pubkey, amount_sats, lock_time_blocks }` |
| Adapter family/chain | `family() -> ChainFamily::BitcoinUtxo`; `chain() -> Chain::Bitcoin` in the current implementation |
| Address input | `validate_address(address)` currently accepts strings beginning with `bc1` |
| Fee input/output | `estimate_fee(&TxParams) -> Result<u64, String>`; current implementation returns `1600` regardless of transaction shape |
| Trust policy | `trust_tier() -> TrustTier::Strict` |
| Proof input | `verify_state_proof(state_root, proof)`; `[height]:[sig_hex]` is an intended/example fixture only. The current adapter accepts any non-empty string containing `:` unless it contains `invalid`; it does not parse the height or decode/verify a signature. |
| Proof output | `Ok(true)`/`Ok(false)` from structural checks; this is not EOTS signature verification |
| State root | `get_state_root() -> "babylon_finality_root"` in the current adapter; it is not a live chain query |
| Signing output | `SignResponse { signature, verification_key, address, derivation }` from a concrete downstream signer, if capability-gated support exists |

## Ownership

| Owner | Owns | Does not own |
| --- | --- | --- |
| Core (`lib-conxian-core`) | Staking intent representation, Bitcoin-family taxonomy, adapter contracts, trust-tier metadata, and structural input checks | BTC headers, EOTS cryptography, private keys, staking transaction builders, RPC, persistence, or retries |
| Conxius Enclave SDK / Wallet | Key custody, derivation, hardware-backed signing, user approval, and concrete Bitcoin transaction construction | Babylon observation, finality evidence acquisition, or Gateway workflow state |
| Gateway (`conxian-gateway`) | Staking workflow orchestration, provider/federation coordination, persistence, retries, and network side effects | Signing key custody and substituting a structural check for cryptographic verification |
| Nexus (`conxian-nexus`) | BTC-header observation, Babylon proof/EOTS acquisition, light-client or verifier backends, and evidence provenance | Signing, staking custody, transaction construction, or runtime retry policy |

## Retryable versus terminal failures

**Potentially retryable or reconcilable downstream failures:**

- temporary header, proof-provider, node, or network unavailability;
- an observation timeout before a staking state is known;
- signer or Gateway coordination failure where no transaction/signature side
  effect has been confirmed.

**Terminal for the supplied input or evidence:**

- empty or malformed staking parameters after application validation;
- invalid address shape, unsupported capability, or invalid UCS request;
- empty proof or proof without the required structural separator;
- cryptographic EOTS/header verification failure once Nexus performs it;
- a conflicting or stale evidence record that fails the downstream verifier
  policy.

Core does not decide whether a timeout is safe to retry. The concrete signer,
Gateway, and Nexus must reconcile external side effects first.

## Fail-closed boundaries

- `TrustTier::Strict` is metadata and policy input; it does not make the
  adapter's proof implementation a light client.
- A proof string that passes the current non-empty/colon checks is not a verified
  EOTS signature. Nexus must perform the cryptographic check and bind it to the
  correct BTC header, height, and finality context.
- The current constant state root must not be treated as live evidence.
- A Bitcoin-family mapping or `Chain::Bitcoin` adapter identity must not be
  used to infer Babylon-specific signing capability.
- Missing signer capability, malformed response, or unverifiable proof blocks
  the flow rather than falling back to a generic Bitcoin signer.

## Current gaps / unsupported behavior

- No Babylon-specific `UniversalChainSigner` or staking transaction builder is
  present in Core.
- BTC-header acquisition, EOTS cryptographic verification, checkpoint/finality
  policy, and network submission are downstream responsibilities.
- Address validation, fee estimation, and state-root output are deliberately
  minimal adapter primitives rather than complete Babylon semantics.
- Core does not own staking persistence, unbonding timers, or retry/recovery
  workflows.

## Source links

- [Universal signing architecture](../SIGNING_ARCHITECTURE.md)
- [UCS contract and types](../../src/signing.rs)
- [Babylon adapter](../../src/babylon/mod.rs)
- [Generic adapter contract](../../src/adapters/mod.rs)
- [Chain and family mapping](../../src/control_model/trust.rs)
- [Protocol verifier ownership](../architecture/PROTOCOL_VERIFIER.md)
- [Core/Gateway boundary](../ARCHITECTURE_BOUNDARIES.md)
