//! # lib-conxian-core
//!
//! Shared protocol primitives for the Conxian ecosystem.
//!
//! ## Relationship to Other Crates
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `conxius-enclave-sdk` | **Production Vault SDK** - Hardware-backed signing, attestation, FROST DKG, BitVM2 |
//! | `lib-conxian-core` | Shared protocol primitives - control models, anchoring, chain types |
//!
//! ## Core Modules
//!
//! - `control_model`: Trust tiers (CON-791), lifecycle states, invariant validation
//! - `anchoring`: State root persistence models
//! - `adapters`: Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint)
//! - `verifier`: Platform-neutral proof, block-reference, finality, and capability contracts
//! - `contract_bridge`: Clarity contract interfaces for Stacks
//!
//! ## SDK Features
//!
//! Enable the `enclave` feature for Vault SDK re-exports:
//!
//! ```toml
//! lib-conxian-core = { version = "0.2", features = ["enclave"] }
//! ```
//!
//! ## Vault SDK Migration
//!
//! For Vault SDK features (hardware-backed signing, MuSig2, BitVM2), use
//! [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) directly.
//! See [docs/MIGRATION.md](docs/MIGRATION.md) for migration instructions from v0.2.x.
//!
//! ## Contact
//!
//! - Support: support@conxian-labs.com
//! - Security: security@conxian-labs.com
//! - Labs: https://www.conxian-labs.com

pub mod adapters;
pub mod anchoring;
pub mod babylon;
pub mod cjcs;
pub mod contract_bridge;
pub mod control_model;
pub mod deployment;
pub mod fedimint;
pub mod protocol;
pub mod verifier;

// CXIP 20 Modular Architecture
pub mod bitcoin;
pub mod crypto;
pub mod enclave;
pub mod lightning;
pub mod rgb;
pub mod signing;
pub mod stacks;

#[cfg(test)]
mod tests;

// Re-export contract bridge types
pub use contract_bridge::{ClarityCall, ContractBridge, SignedContractCall};

// Re-export the platform-neutral protocol verification contract and models.
pub use verifier::{
    compute_evidence_binding_hash, validate_finality_result, validate_finality_result_at,
    validate_finality_transition, validate_proof_envelope, validate_proof_envelope_at,
    validate_proof_verification_result, validate_proof_verification_result_at, BlockHeader,
    BlockReference, CapabilityAdvertisement, ChainId, ChainStateReference,
    ChainStateVerificationRequest, DynProtocolVerifier, LatestVerifiedBlock, ProofData,
    ProofFormat, ProofVerificationRequest, ProofVerificationResult, ProtocolVerifier,
    ProtocolVerifierBackend, ProtocolVerifierError, TransactionFinalityRequest,
    TransactionFinalityResult, TransactionFinalityStatus, VerificationProvenance,
    VerifiedBlockReference, VerifierCapabilities, VerifierCapability,
    PROTOCOL_VERIFIER_EVIDENCE_BINDING_DOMAIN, PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION,
};

// Re-export Vault SDK primitives when enclave feature is enabled
#[cfg(feature = "enclave")]
pub use conxius_enclave_sdk::enclave::{
    EnclaveManager, SignRequest, SignResponse, SigningAlgorithm,
};
#[cfg(feature = "enclave")]
pub use conxius_enclave_sdk::{ConclaveError, ConclaveResult};

#[cfg(test)]
mod deployment_tests {
    use super::deployment::*;

    #[test]
    fn test_deployment_plan_agent_readable() {
        let mut plan = DeploymentPlan::new("test-proj", "1.0.0");
        plan.add_contract("token-contract", "sha256:abc");
        let readable = plan.to_agent_readable();
        assert!(readable.contains("test-proj"));
        assert!(readable.contains("token-contract"));
        assert!(readable.contains("nakamoto_integrity_hash"));
    }
}
