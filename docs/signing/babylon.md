# Babylon signing flow

## Scope and current support status

This guide covers the Babylon Bitcoin-staking handoff: staking intent data,
Bitcoin transaction signing, BTC-header observation, and Babylon EOTS evidence.
Core supplies the `StakingIntent` DTO/protocol model and a `BabylonAdapter`
implementing the generic adapter interface. Gateway/downstream code constructs,
policy-validates, persists, and advances staking intents. Core does not own that
construction, storage, or policy workflow, and it does not create
staking/delegation/unbonding or withdrawal transactions, own EOTS keys, verify
EOTS signatures in production, run a light client, or broadcast Bitcoin
transactions.

The `BabylonAdapter` support described here is structural only; it does not
establish BTC-header, checkpoint, or EOTS verification.

The v0.3.0 boundary is intentionally fail-closed: structural Babylon input is
not an authorization result, and the typed adapter errors do not replace a
Nexus-backed verifier.

`BabylonAdapter::verify_state_proof` does not implement BTC header, height,
EOTS signature, checkpoint, or cryptographic commitment verification. It returns
typed `StateProofError::MalformedInput` for empty evidence and
`StateProofError::Unsupported` for otherwise shaped input.

## Canonical target and UCS boundary

There are two identities in a Babylon flow and they must not be silently
collapsed:

- `Chain::Babylon` exists in the Core taxonomy and maps to
  `ChainFamily::BitcoinUtxo`; it is the natural target for a capability that
  explicitly represents Babylon protocol operations.
- `BabylonAdapter::chain()` currently returns `Chain::Bitcoin`, while its
  `family()` is `BitcoinUtxo`. That implementation mismatch means an adapter
  result cannot be treated as a Babylon-specific signer capability without an
  explicit contract decision.

For actual Bitcoin staking/delegation/unbonding/withdrawal transaction bytes,
the transaction signer may instead advertise
`SigningTarget::for_chain(Chain::Bitcoin)`. The capability declaration must
make the chosen identity explicit; `SignerCapabilities::require` will not
coerce `Chain::Babylon` and `Chain::Bitcoin` for the caller.

The UCS boundary remains an explicit `SignRequest`/`SignResponse` exchange. Core
validates the target, algorithm, payload, derivation context, and advertised
capability. The enclave SDK owns the private key and concrete signing
implementation. Babylon message encoding, Bitcoin transaction construction,
and EOTS proof production are outside the Core signer contract.

## Participant ownership

| Participant | Owns in this flow | Does not own |
| --- | --- | --- |
| Core | The `StakingIntent` DTO/protocol model, UCS target/capability validation, trust/finality DTOs, and structural adapter contracts | Constructing, policy-validating, or persisting staking intents; EOTS key custody, Bitcoin transaction construction, headers, light clients, or broadcast |
| `conxius-enclave-sdk` | Hardware-backed staking-key custody, Bitcoin signing, attestation, and any concrete UCS implementation | Babylon observation or Gateway workflow state |
| Gateway | Staking-intent construction and policy validation, lifecycle, provider selection, signer coordination, persistence, retries, and Bitcoin/Babylon side effects | EOTS cryptographic truth or private-key custody |
| Nexus | BTC headers, checkpoints, Babylon observations, EOTS verification backends, and finality evidence | Signing keys, user approval, or workflow retries |
| Wallet | User review/approval of amount, lock time, provider, and fee policy | Header/EOTS verification or bridge orchestration |

## Sequence and ownership

| Step | Owner | Inputs and evidence | Core contract / boundary | Output | Stop condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Gateway | Staker public key, finality-provider public key, amount, lock time | Core supplies the `StakingIntent` DTO/model; Gateway/downstream owns construction, policy validation, persistence, and the explicit Babylon or Bitcoin signing identity | Gateway-owned reviewable staking intent | Missing keys, zero/invalid amount, or invalid lock policy |
| 2 | Gateway + transaction adapter | UTXOs, staking script, checkpoint/delegation fields, Bitcoin sighash context | Core does not build the transaction; it carries only the explicit signing payload and metadata | Unsigned Bitcoin transaction bytes | Unsupported script context or unclassified BIP-110 surface |
| 3 | Wallet + SDK | `SignRequest`, capability advertisement, attestation/policy evidence | UCS fail-closed request/response validation | `SignResponse` for the actual Bitcoin transaction | Target mismatch, unsupported operation, malformed payload, or backend failure |
| 4 | Nexus | Transaction id, block header chain, confirmations, proof provenance | `ProtocolVerifier` validates state/finality evidence around a Nexus backend | Verified Bitcoin header/finality result | Stale/malformed proof, unverified header, evidence mismatch, or non-final required state |
| 5 | Nexus | BTC header context, EOTS proof, checkpoint/evidence binding | Core can carry `ProofData` and policy metadata; the façade does not implement EOTS math | Verified Babylon evidence or typed failure | No production EOTS verifier, invalid signature, wrong height, wrong checkpoint, or policy block |
| 6 | Gateway | Verified evidence, signer/provider acknowledgements, expiry/unbonding policy | Core state/invariant types remain the contract; Gateway owns persistence and side effects | Staking status and, later, withdrawal/unbonding transaction request | Never finalize or release from `StateProofError::Unsupported` or structural input alone |

## Required inputs

- Explicit target identity: `Chain::Babylon` for Babylon capability metadata or
  `Chain::Bitcoin` for actual Bitcoin transaction signing, with the selected
  identity advertised by the signer.
- A Gateway/downstream-constructed `StakingIntent` with staker key,
  finality-provider key, amount, and lock time; Core supplies the DTO/model but
  does not validate or persist the staking policy.
- Fully constructed Bitcoin transaction bytes and the parsed context required
  by the downstream adapter to classify BIP-110 surfaces.
- BTC header/proof data, checkpoint reference, EOTS proof, provenance, and
  finality policy supplied by Nexus/Gateway.
- User approval and attestation/policy evidence owned by Wallet and the SDK.

## Required outputs

- A validated `SignResponse` for each actual Bitcoin transaction surface.
- A Nexus-produced `ProtocolVerifier` result for Bitcoin state/finality and a
  separately verified Babylon EOTS/checkpoint result when the operation needs
  it.
- A Gateway-owned staking lifecycle record. Core's `StakingIntent` is not a
  broadcast receipt or a finality certificate.

## Verification and finality boundary

BTC headers, chain history, checkpoint binding, and EOTS proof verification are
Nexus responsibilities. The Babylon adapter's proof boundary is structural
only, so it cannot substitute for those checks. Gateway consumes them through
a `ProtocolVerifier` façade backed by a concrete Nexus verifier. The façade
checks advertised chain,
proof format, state/block identity, evidence binding, provenance, trust tier,
verification class, finality class, and verifier identity. It does not implement
the EOTS cryptographic verifier.

`BabylonAdapter` is not a substitute: its `chain()` returns `Chain::Bitcoin`
and its proof check is unavailable without a downstream verifier. A production `Strict` result must meet
the verifier capability policy, including the required light-client class where
applicable. BTC finality for a staking transaction must be evaluated on the
Bitcoin transaction/header evidence, not inferred from a Babylon intent or a
non-empty proof string.

BIP-110 applies to actual Bitcoin staking/delegation/unbonding/withdrawal or
checkpoint transaction bytes. BTC header observations and EOTS/Babylon
messages are off-chain evidence and must not be relabeled as Bitcoin pushdata
or witness elements merely because the protocol is Bitcoin-anchored.

## Retry versus terminal semantics

- **Potentially retryable or waitable:** delayed Bitcoin confirmations,
  temporarily unavailable headers or Nexus provider, delayed EOTS evidence,
  or a pending staking lifecycle state while a configured observation window is
  still open.
- **Terminal:** target identity mismatch, unsupported capability, malformed
  staking intent or signing payload, invalid signer response, malformed or
  cryptographically invalid EOTS/header evidence, checkpoint mismatch, expired
  evidence, verifier identity mismatch, policy block, or required finality not
  reached before an irreversible deadline.

Operational retry classification is downstream policy owned by Gateway. Core
does not poll headers, retry EOTS verification, or downgrade an invalid proof to
an observation-only success.

## Fail-closed boundaries

- Do not silently map `Chain::Babylon` to `Chain::Bitcoin` or accept the
  `BabylonAdapter::chain()` mismatch as a production capability declaration.
- Do not accept a structurally formatted EOTS proof as a verified EOTS
  signature.
- Require Bitcoin header/finality evidence from a policy-compatible verifier
  before staking, unbonding, withdrawal, or checkpoint-dependent effects.
- Apply BIP-110 checks to actual Bitcoin transaction surfaces with an enabled
  compliance configuration; do not classify off-chain Babylon messages as
  Bitcoin data surfaces.
- Never release funds or advance a terminal lifecycle state after any failed
  signer, proof, checkpoint, policy, or finality precondition.

## Known gaps and unsupported behavior

- No production Babylon signer, staking transaction builder, EOTS key/proof
  implementation, Babylon light client, checkpoint verifier, or broadcast path
  exists in Core.
- `BabylonAdapter::verify_state_proof` is unavailable without a downstream EOTS
  and header verifier, and `get_state_root` returns a typed unavailable error;
  it is not a production EOTS verifier.
- `BabylonAdapter::chain() == Chain::Bitcoin` conflicts with the distinct
  `Chain::Babylon` taxonomy variant; downstream capability and verifier
  contracts must resolve that identity explicitly.
- Core does not define Babylon-specific transaction DTOs or lifecycle
  persistence.

## Source references

- UCS contract: [`src/signing.rs`](../../src/signing.rs) and
  [`docs/SIGNING_ARCHITECTURE.md`](../SIGNING_ARCHITECTURE.md)
- Babylon adapter and intent: [`src/babylon/mod.rs`](../../src/babylon/mod.rs)
- Chain mapping and trust tiers: [`src/control_model/trust.rs`](../../src/control_model/trust.rs)
- Protocol verification: [`src/verifier.rs`](../../src/verifier.rs) and
  [`docs/architecture/PROTOCOL_VERIFIER.md`](../architecture/PROTOCOL_VERIFIER.md)
- BIP-110 Bitcoin-surface boundary: [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md)
- Architecture boundaries: [`docs/ARCHITECTURE_BOUNDARIES.md`](../ARCHITECTURE_BOUNDARIES.md)
- Downstream contracts: [enclave SDK issue #179](https://github.com/Conxian/conxius-enclave-sdk/issues/179),
  [Gateway issue #245](https://github.com/Conxian/conxian-gateway/issues/245),
  [Nexus issue #163](https://github.com/Conxian/conxian-nexus/issues/163), and
  [Wallet issue #381](https://github.com/Conxian/conxius-wallet/issues/381)
