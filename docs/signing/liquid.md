# Liquid signing flow

## Scope and current support status

This guide covers Liquid/Elements peg-in and peg-out ownership, sidechain
signing handoffs, and the boundary between Core's fail-closed adapter boundary
and production federation/proof logic. Core currently exposes `LiquidAdapter`
with Bitcoin-family mapping, prefix/length address checks, a fee stub, and a
typed unsupported proof boundary. It does not contain Liquid peg DTOs, federation
quorum/signing logic, an Elements transaction builder, confidential-asset proof
verification, RPC clients, or a production Liquid signer.

The adapter's address checks are structural only and do not establish a valid
Liquid transaction, peg, federation quorum, or finality result.

Liquid sidechain state and a Bitcoin L1 peg transaction are different signing
surfaces. The flow must not use a Liquid address or sidechain proof as a proxy
for Bitcoin peg finality.

## Canonical target and UCS boundary

For a Liquid-side operation, the canonical UCS target is
`SigningTarget::for_chain(Chain::Liquid)` with `ChainFamily::Federation`.
Core provides `AddressFormat::LiquidConfidential` and `AddressFormat::FederationAddress`;
a concrete signer must advertise one of these or `AddressFormat::Generic` only if it owns
the real Elements address validation. For a Bitcoin L1 peg transaction, use a separate
`SigningTarget::for_chain(Chain::Bitcoin)` request.

The UCS boundary carries the exact bytes or digest for the selected surface:

1. Wallet/Gateway choose the target and construct a secret-free `SignRequest`.
2. `SignerCapabilities::require` and `UniversalChainSigner::sign` validate the
   target, algorithm, operation, payload, and derivation context.
3. The enclave SDK returns a public `SignResponse`; Core does not receive a
   private key, seed, share, or enclave handle.

Core does not infer Elements serialization, blinding factors, asset issuance,
fee rules, federation threshold policy, or peg script semantics from the
`SigningPayload`.

## Participant ownership

| Participant | Owns in this flow | Does not own |
| --- | --- | --- |
| Core | Liquid target/family mapping, UCS DTO/capability validation, structural adapter contract, and verifier/finality models | Federation custody, Elements/confidential transaction construction, peg execution, or network I/O |
| `conxius-enclave-sdk` | Hardware-backed key custody, signer implementation, federation/member signing where contracted, and attestation | Peg orchestration and sidechain observation |
| Gateway | Peg-in/out workflow, federation/provider coordination, persistence, retry policy, release/broadcast effects, and policy routing | Private-key custody or replacement of Core validation |
| Nexus | Liquid/Elements and Bitcoin observations, proof acquisition, chain-state/finality verifier backends | Signing keys and user approval |
| Wallet | User approval, destination/amount/fee review, and display of peg status | Federation quorum, proof verification, or transaction construction truth |

## Sequence and ownership

| Step | Owner | Inputs and evidence | Core contract / boundary | Output | Stop condition |
| --- | --- | --- | --- | --- | --- |
| 1 | Wallet + Gateway | Direction, amount, Liquid/Bitcoin destination, fee and federation policy | Select `Chain::Liquid` for sidechain bytes or `Chain::Bitcoin` for L1 bytes | Reviewable intent and target | Missing/invalid destination, amount, or policy |
| 2 | Gateway + downstream Elements/Bitcoin adapter | UTXOs, peg script, Elements inputs/outputs, blinding/asset context, sighash bytes | Core does not build either transaction; it accepts explicit signing bytes and metadata | Unsigned transaction surface | Unsupported serialization, missing confidential context, or unclassified Bitcoin BIP-110 surface |
| 3 | Wallet + SDK | `SignRequest`, capability advertisement, attestation, federation/member policy | UCS validates request/response metadata and fails closed | `SignResponse` | Unsupported capability, malformed payload, backend failure, or invalid response |
| 4 | Nexus | Liquid block/proof, federation/peg evidence, Bitcoin txid and headers where applicable | `ProtocolVerifier` validates the evidence envelope and policy metadata around a real backend | Verified or non-final evidence | Structural-only proof, stale/malformed proof, identity mismatch, or policy block |
| 5 | Gateway | Verified evidence, signer quorum, provider response, timelock/release policy | Core supplies typed inputs; Gateway owns persistence and side effects | Peg-in/out status and release decision | Missing quorum, failed proof, failed policy, or missing Bitcoin finality |
| 6 | Nexus | Sidechain and/or Bitcoin finality result | Core validates the finality result; it does not declare settlement from an intent | Finalized or pending operational record | Never finalize from an address prefix, static root, or structural proof alone |

## Required inputs

- An explicit Liquid or Bitcoin signing target matching the bytes being signed.
- A capability advertisement covering the chosen chain, algorithm, operation,
  and address representation.
- Fully constructed Elements/Bitcoin bytes or a correctly sized digest; Core
  does not supply confidential transaction serialization or peg scripts.
- For Liquid proof decisions, the actual block/proof/evidence format and
  provenance required by a Nexus verifier backend.
- For a Bitcoin-side peg, parsed transaction context for BIP-110 size checks
  and a Bitcoin finality request when policy requires it.
- Wallet approval, SDK attestation, federation/provider acknowledgements, and
  Gateway policy context.

## Required outputs

- A validated `SignResponse` for each Liquid or Bitcoin transaction surface.
- A `ProtocolVerifier` result whose chain identity, proof/result binding,
  provenance, trust tier, verification class, and finality class satisfy policy.
- A Gateway-owned peg lifecycle and release record. Core does not mint, burn,
  release, or broadcast assets.

## Verification and finality boundary

Nexus owns chain observation and any real Liquid/Elements or Bitcoin verifier
backend. Gateway must consume that backend through `ProtocolVerifier`, which
validates capabilities, request/result identity, state roots, provenance,
evidence binding, trust policy, and finality requirements. A finality request
that requires finality must not be satisfied by a merely observed block.

The current `LiquidAdapter::verify_state_proof` rejects empty input as
`StateProofError::MalformedInput` and returns typed
`StateProofError::Unsupported` for non-empty input. Its `get_state_root` result
is typed `StateProofError::Unavailable`. `LiquidAdapter::validate_address`
still checks only `ex1`/`tlq1` prefix and minimum length. These APIs are not
confidential proof, Merkle proof, Elements consensus, or federation verification.

BIP-110 applies to a Bitcoin L1 peg transaction only when its actual outputs,
scripts, and applicable witness elements are serialized on Bitcoin. Liquid
sidechain transactions and confidential proofs are not Bitcoin pushdata merely
because the system is Bitcoin-family mapped.

## Retry versus terminal semantics

- **Potentially retryable or waitable:** delayed Liquid/Bitcoin observations,
  temporarily unavailable proof/federation provider, incomplete signer quorum,
  or a non-final transaction while policy permits additional confirmations.
- **Terminal:** target/capability mismatch, malformed transaction bytes,
  invalid signer response, malformed or structurally invalid proof, failed
  confidential/federation verification, evidence-binding mismatch, stale or
  expired evidence, policy block, or a required finality failure that cannot be
  waited through.

Operational retry classification is downstream policy owned by Gateway. Core
does not retry federation members, poll sidechain nodes, or turn a typed
unsupported result into cryptographic proof.

## Fail-closed boundaries

- Keep Liquid-side and Bitcoin-L1 signing targets separate.
- Do not treat `ex1`/`tlq1` prefix validation as complete address validation.
- Do not treat `StateProofError::Unsupported` as verification of Merkle,
  confidential-asset, federation, or Elements consensus evidence.
- Require a policy-compatible `ProtocolVerifier` result and required Bitcoin
  finality before a peg release or irreversible side effect.
- Apply BIP-110 only to parsed Bitcoin surfaces with an enabled compliance
  configuration; reject unknown/unclassified transaction context.
- Stop after any failed signer, quorum, proof, policy, or finality precondition.

## Known gaps and unsupported behavior

- Core defines no peg-in/peg-out DTOs, federation threshold protocol, Elements
  transaction builder, confidential proof implementation, or Liquid broadcast
  integration.
- `LiquidAdapter` address checks are structural, proof verification is typed
  unsupported, and its state root is unavailable; it is not a production
  verifier.
- No production Liquid `UniversalChainSigner` or
  `ProtocolVerifierBackend` is implemented in this repository.
- The coarse `BitcoinUtxo` family mapping does not provide Liquid transaction
  semantics or imply that Bitcoin and Liquid transactions are interchangeable.

## Source references

- UCS contract: [`src/signing.rs`](../../src/signing.rs) and
  [`docs/SIGNING_ARCHITECTURE.md`](../SIGNING_ARCHITECTURE.md)
- Liquid adapter: [`src/bitcoin/liquid_adapter.rs`](../../src/bitcoin/liquid_adapter.rs)
- Chain-family mapping: [`src/control_model/trust.rs`](../../src/control_model/trust.rs)
- Protocol verification: [`src/verifier.rs`](../../src/verifier.rs) and
  [`docs/architecture/PROTOCOL_VERIFIER.md`](../architecture/PROTOCOL_VERIFIER.md)
- BIP-110 Bitcoin-surface boundary: [`src/control_model/bip110.rs`](../../src/control_model/bip110.rs)
  and [`docs/BIP110_ALIGNMENT.md`](../BIP110_ALIGNMENT.md)
- Architecture boundaries: [`docs/ARCHITECTURE_BOUNDARIES.md`](../ARCHITECTURE_BOUNDARIES.md)
- Downstream contracts: [enclave SDK issue #179](https://github.com/Conxian/conxius-enclave-sdk/issues/179),
  [Gateway issue #245](https://github.com/Conxian/conxian-gateway/issues/245),
  [Nexus issue #163](https://github.com/Conxian/conxian-nexus/issues/163), and
  [Wallet issue #381](https://github.com/Conxian/conxius-wallet/issues/381)
