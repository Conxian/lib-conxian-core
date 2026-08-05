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
//! - `control_model`: Trust tiers (CON-791), lifecycle states, versioned static risk profiles, and invariant validation
//! - `anchoring`: State root persistence models
//! - `adapters`: Universal chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint, +15 family adapters)
//! - `verifier`: Platform-neutral proof, block-reference, finality, and capability contracts
//! - `contract_bridge`: Clarity contract interfaces for Stacks
//! - `sdk`: **Comprehensive SDK re-exports** — all 70 accessible conxius-enclave-sdk modules organized by category (Session 52)
//!
//! ## SDK Features (Session 52 — Full Alignment)
//!
//! Enable the `full-sdk` feature for access to all 70 accessible SDK modules:
//!
//! ```toml
//! lib-conxian-core = { version = "0.3", features = ["full-sdk"] }
//! ```
//!
//! Or enable individual categories:
//!
//! ```toml
//! lib-conxian-core = { version = "0.3", features = ["sdk-blockchain", "sdk-cross-cutting"] }
//! ```
//!
//! Then access via `conxian_core::sdk::*`:
//!
//! ```rust,ignore
//! use conxian_core::sdk::blockchain::{bitcoin, statechain, dlc};
//! use conxian_core::sdk::cross_cutting::{intent, settlement, economy};
//! use conxian_core::sdk::rails::{bisq, wormhole};
//! ```
//!
//! ## Vault SDK Migration
//!
//! For Vault SDK features (hardware-backed signing, MuSig2, BitVM2), use
//! [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) directly
//! OR enable `full-sdk` on lib-conxian-core for re-exported access.
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
pub mod chain;
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

// ── SDK re-exports (Session 52: all 70 accessible modules) ──
#[cfg(any(
    feature = "enclave",
    feature = "sdk-blockchain",
    feature = "sdk-cross-cutting",
    feature = "sdk-rails",
    feature = "sdk-nexus",
    feature = "sdk-infrastructure",
))]
pub mod sdk;

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
