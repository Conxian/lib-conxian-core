# Protocol Verifier Architecture

`lib-conxian-core` exposes the platform-neutral `ProtocolVerifier` contract in
`src/verifier.rs`. The contract describes what a verifier can prove; it does not
implement proof acquisition, chain observation, persistence, or live routing.

## Ownership boundary

| Layer | Owns | Does not move into Core |
| --- | --- | --- |
| **Core** (`lib-conxian-core`) | `ProtocolVerifier`, `ChainId`, proof inputs/results, latest verified block references, transaction finality statuses, capability advertisements, typed errors, and invariant helpers | RPC/light-client clients, network I/O, storage, polling, retries, or chain-specific proof backends |
| **Enclave SDK** (`conxius-enclave-sdk`) | Hardware-backed signing, attestation, and enclave policy primitives that can supply trusted evidence to a downstream verifier | The Core contract's runtime orchestration or a new Core-side transport layer |
| **Nexus** (`conxian-nexus`) | Chain observation, proof acquisition, chain-specific light clients, and verifier implementations for supported rails | Changes to Core's canonical trust/finality taxonomies for one provider |
| **Gateway** (`conxian-gateway`) | Request orchestration, persistence, provider selection, retries, policy routing, and external side effects | Network or database behavior hidden inside Core types |

No runtime verifier implementation is introduced by this contract work. Nexus,
Gateway, or a downstream adapter may implement the trait when it has a concrete
evidence source.

## Contract examples

### 1. Advertise capabilities before accepting work

```rust
use lib_conxian_core::control_model::ChainFamily;
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
    verification_classes: vec![
        lib_conxian_core::control_model::VerificationClass::LightClient,
    ],
    finality_classes: vec![],
    trust_tiers: vec![lib_conxian_core::control_model::TrustTier::Strict],
};

capabilities
    .require_proof_format(&chain, &ProofFormat::HeaderChain)
    .expect("advertised capability");
```

An unsupported chain, capability, or proof format returns a typed error rather
than being treated as a successful verification.

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

Core checks structural sufficiency, envelope expiry, existing trust-tier
policy, and malformed metadata. Cryptographic proof verification remains in the
downstream implementation.

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

Implementations should validate their capability advertisement and request
before acquiring or interpreting evidence. The typed error taxonomy distinguishes
unsupported chains/capabilities, malformed or insufficient proofs, invalid or
unavailable evidence, expired evidence, stale references, non-final state, and
policy-blocked trust mappings. `ObserverOnly` remains non-production, and
`Strict` continues to require the existing `LightClient` verification class.
