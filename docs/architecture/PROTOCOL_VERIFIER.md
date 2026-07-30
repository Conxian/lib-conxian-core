# Protocol Verifier Architecture

`lib-conxian-core` exposes an enforceable, platform-neutral verifier façade in
`src/verifier.rs`. `ProtocolVerifier<B>` owns the consumer-facing methods;
`ProtocolVerifierBackend` is the lower-level implementation-hook trait used by
Nexus, Gateway, or a downstream adapter. Backends cannot override the façade
methods or skip their shared checks.

## Ownership boundary

| Layer | Owns | Does not move into Core |
| --- | --- | --- |
| **Core** (`lib-conxian-core`) | `ProtocolVerifier<B>`, `ProtocolVerifierBackend`, `ChainId`, proof inputs/results, evidence-binding encoding, latest verified block references, transaction finality statuses, capability advertisements, typed errors, and invariant helpers | RPC/light-client clients, network I/O, storage, polling, retries, or chain-specific proof backends |
| **Enclave SDK** (`conxius-enclave-sdk`) | Hardware-backed signing, attestation, and enclave policy primitives that can supply trusted evidence to a downstream verifier | The Core contract's runtime orchestration or a new Core-side transport layer |
| **Nexus** (`conxian-nexus`) | Chain observation, proof acquisition, chain-specific light clients, and verifier backends for supported rails | Changes to Core's canonical trust/finality taxonomies for one provider |
| **Gateway** (`conxian-gateway`) | Request orchestration, persistence, provider selection, retries, policy routing, and external side effects | Network or database behavior hidden inside Core types |

Core provides the contract and structural checks only. The façade does not
prove cryptographic authenticity; downstream signatures, attestations,
light-client proofs, or verifier-set proofs remain required.

## Enforceable façade and backend hooks

Implementations provide lower-level hooks and are wrapped before consumers use
them:

```rust,ignore
use lib_conxian_core::verifier::{
    ProofVerificationRequest, ProofVerificationResult, ProtocolVerifier,
    ProtocolVerifierBackend, ProtocolVerifierError, VerifierCapabilities,
};

struct NexusBackend {
    capabilities: VerifierCapabilities,
}

impl ProtocolVerifierBackend for NexusBackend {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        // Acquire and verify chain-specific evidence here.
        todo!()
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &lib_conxian_core::verifier::ChainId,
    ) -> Result<lib_conxian_core::verifier::LatestVerifiedBlock, ProtocolVerifierError> {
        todo!()
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &lib_conxian_core::verifier::TransactionFinalityRequest,
    ) -> Result<lib_conxian_core::verifier::TransactionFinalityResult, ProtocolVerifierError> {
        todo!()
    }
}

let capabilities: VerifierCapabilities = todo!();
let request: ProofVerificationRequest = todo!();
let verifier = ProtocolVerifier::try_new(NexusBackend { capabilities })?;
let result = verifier.verify_chain_state(&request)?;
```

Every façade operation performs the following sequence:

1. Validate the capability advertisement and request before invoking a backend
   hook.
2. Invoke exactly the lower-level backend hook for the requested operation.
3. Validate result structure, chain/block/proof identity, state-root
   postconditions, provenance timestamps, advertised trust/verification/finality
   policy membership, verifier identity, and finality policy before returning
   success. This applies to nested latest-block metadata returned with a
   finality result as well.

`ProtocolVerifier::new` keeps invalid advertisements representable but fails
closed on the first operation. `ProtocolVerifier::try_new` rejects an invalid
advertisement immediately. For runtime-selected implementations, use the
`DynProtocolVerifier` alias (`ProtocolVerifier<Box<dyn ProtocolVerifierBackend>>`).

## Canonical chain identity

`ChainId::from_chain` derives its family from the shared
`control_model::chain_family_for` mapping. `ChainId::try_from_parts` checks
explicit parts, and deserializing a known chain with a mismatched family fails.
The taxonomy assigns each chain to its canonical family per bitcoinlayers.org:
Lightning stays in `BitcoinUtxo`, Stacks maps to `Anchor`, Liquid to `Federation`,
Babylon to `BPoS`, etc. Concrete chain capabilities still distinguish their
operations and address/proof formats.

## Request-aware proof result checks

`validate_proof_verification_result_at` requires the returned result to match
the request's:

- chain identity;
- block hash and height;
- proof format; and
- requested state root, including presence in both the result state reference
  and verified block header, with exact equality.

An omitted or changed requested state root returns a typed
`MissingStateRoot` or `MismatchedStateRoot` error. A backend returning a result
that bypasses all helper validation therefore cannot turn an invalid response
into façade-level success.

## Result policy postconditions

The façade validates every returned policy triple against the immutable
`VerifierCapabilities` snapshot captured at construction:

- `trust_tier` must appear in `capabilities.trust_tiers`;
- `verification_class` must appear in `capabilities.verification_classes`; and
- `finality_class` must appear in `capabilities.finality_classes`.

The returned provenance `verifier_id` must also equal
`capabilities.verifier_id`. These checks run for state-proof results, latest
verified blocks, finality results, and any nested latest block carried by a
finality result. Mismatches return typed errors rather than being treated as
verified evidence.

## Evidence timestamps and provenance

`validate_proof_envelope_at` uses deterministic half-open semantics:

```text
observed_at <= now < expires_at
```

Future observations return `FutureDatedEvidence`; an expiry equal to `now` is
already expired; `expires_at <= observed_at` is malformed. Verification
provenance follows the same no-future policy: `verified_at` must be no later
than the validation time. Callers that need reproducible tests should use the
`*_at` methods.

## Structural evidence binding

When a `ProofEnvelope` is present, `compute_evidence_binding_hash` calculates a
versioned, domain-separated SHA-256 digest over canonical request, proof, and
envelope fields. The encoding uses explicit field order, type tags, and
length-prefixing; JSON objects are sorted by key before encoding. The mirrored
`ProofData.evidence_hash` and `ProofEnvelope.evidence_hash` fields are the only
fields excluded because they carry the digest itself. The envelope destination
must equal the request's canonical chain ID, and both hash fields must equal
the computed digest.

This hash provides structural integrity and consistency between the DTOs. It
does **not** provide authenticity. Downstream signatures, attestation, light
clients, or verifier-set proofs are still required before treating evidence as
trusted.

## Contract examples

### 1. Advertise capabilities before accepting work

```rust
use lib_conxian_core::control_model::{ChainFamily, TrustTier, VerificationClass};
use lib_conxian_core::verifier::{
    ChainId, ProofFormat, VerifierCapabilities, VerifierCapability,
};

let chain = ChainId::new(ChainFamily::BitcoinUtxo, "mainnet");
let capabilities = VerifierCapabilities {
    verifier_id: "nexus-bitcoin".into(),
    version: "1".into(),
    supported_chains: vec![chain.clone()],
    supported_families: vec![ChainFamily::BitcoinUtxo],
    capabilities: vec![VerifierCapability::StateProofVerification],
    proof_formats: vec![ProofFormat::HeaderChain],
    verification_classes: vec![VerificationClass::LightClient],
    finality_classes: vec![],
    trust_tiers: vec![TrustTier::Strict],
};

capabilities
    .require_proof_format(&chain, &ProofFormat::HeaderChain)
    .expect("advertised capability");
```

An unsupported chain, capability, or proof format returns a typed error rather
than being treated as successful verification.

### 2. Validate a proof request without choosing its backend

```rust
use lib_conxian_core::control_model::ChainFamily;
use lib_conxian_core::verifier::{
    ChainId, ChainStateReference, ProofData, ProofFormat, ProofVerificationRequest,
};

let request = ProofVerificationRequest::new(
    ChainId::new(ChainFamily::Evm, "ethereum-mainnet"),
    ChainStateReference::new("0xblock", 19_000_000, Some("0xroot".into())),
    ProofData::new(ProofFormat::Merkle, vec![1, 2, 3]),
);

request.validate().expect("structurally valid proof input");
```

Core checks structural sufficiency, envelope time/policy metadata, evidence
binding when an envelope is present, and request/result consistency.
Cryptographic proof verification remains in the downstream backend.

### 3. Keep finality transitions explicit

```rust
use lib_conxian_core::verifier::{
    validate_finality_transition, TransactionFinalityStatus,
};

let pending = TransactionFinalityStatus::Pending;
let confirmed = TransactionFinalityStatus::Confirmed { confirmations: 2 };
let finalized = TransactionFinalityStatus::Finalized { confirmations: 6 };

assert!(validate_finality_transition(&pending, &confirmed).is_ok());
assert!(validate_finality_transition(&confirmed, &finalized).is_ok());
assert!(validate_finality_transition(&finalized, &pending).is_err());
```

`FinalityClass` describes the chain's finality model (`Economic`,
`Probabilistic`, or `Deterministic`); `TransactionFinalityStatus` describes the
transaction's current lifecycle state. A verifier must not infer finality from
an arbitrary successful proof or from confirmations alone.

## Fail-closed behavior

The typed error taxonomy distinguishes invalid chain families, unsupported
chains/capabilities, malformed or insufficient proofs, invalid or unavailable
evidence, expired or future-dated evidence, missing or mismatched state roots,
binding mismatches, stale references, non-final state, and policy-blocked trust
mappings. `ObserverOnly` remains non-production, and `Strict` continues to
require the existing `LightClient` verification class.
