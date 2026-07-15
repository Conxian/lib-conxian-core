//! # lib-conxian-core
//!
//! Shared protocol primitives for the Conxian ecosystem.
//!
//! > ⚠️ **Deprecation Notice**: This library contains deprecated Vault SDK primitives
//! > that will be removed in v0.3.0. See [docs/MIGRATION.md](docs/MIGRATION.md) for
//! > migration instructions to [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk).
//!
//! ## Relationship to Other Crates
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `conxius-enclave-sdk` | **Production Vault SDK** - Hardware-backed signing, attestation, FROST DKG, BitVM2 |
//! | `lib-conxian-core` | Shared protocol primitives - control models, anchoring, chain types |
//!
//! ## Active Modules (Not Deprecated)
//!
//! - `control_model`: Trust tiers (CON-791), lifecycle states, invariant validation
//! - `anchoring`: State root persistence models
//! - `adapters`: Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint)
//! - `contract_bridge`: Clarity contract interfaces
//!
//! ## Deprecated Modules (Will be removed in v0.3.0)
//!
//! - `sdk_primitive`: Use `conxius-enclave-sdk` instead
//! - `musig2`: Use `conxius_enclave_sdk::protocol::musig2` instead
//! - `bitvm2`: Use `conxius_enclave_sdk::protocol::bitvm2` instead
//! - `wallet`: Use `k256` crate instead
//!
//! ## Contact
//!
//! - Support: support@conxian-labs.com
//! - Security: security@conxian-labs.com
//! - Labs: https://www.conxian-labs.com

pub mod adapters;
pub mod anchoring;
pub mod babylon;
pub mod bitvm2;
pub mod cjcs;
pub mod contract_bridge;
pub mod control_model;
pub mod deployment;
pub mod fedimint;
pub mod musig2;
pub mod protocol;
pub mod sdk_primitive;
pub mod wallet;

// CXIP 20 Modular Architecture
pub mod bitcoin;
pub mod crypto;
pub mod enclave;
pub mod lightning;
pub mod rgb;
pub mod stacks;
#[cfg(test)]
mod tests;

pub use contract_bridge::{ClarityCall, ContractBridge, SignedContractCall};
pub use wallet::Wallet;

// Re-export Vault SDK primitives when enclave feature is enabled
#[cfg(feature = "enclave")]
pub use conxius_enclave_sdk::enclave::{
    EnclaveManager, SignRequest, SignResponse, SigningAlgorithm,
};
#[cfg(feature = "enclave")]
pub use conxius_enclave_sdk::{ConclaveError, ConclaveResult};

// Legacy re-exports (deprecated - use conxius-enclave-sdk directly)
#[deprecated(since = "0.2.10", note = "Use conxius-enclave-sdk crate directly for Vault SDK features")]
pub use sdk_primitive::{SigningPolicy, VaultSDK};

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

#[cfg(test)]
mod bitvm2_orchestration_tests {
    #![allow(deprecated)]
    use super::bitvm2::*;

    #[test]
    fn test_segment_generation() {
        let orchestrator = Bitvm2Orchestrator::new();
        let segments = orchestrator.generate_segments("0xabc");
        assert_eq!(segments.len(), 364);
        assert_eq!(segments[0].segment_index, 0);
        assert!(segments[0].script_hash.contains("0xabc"));
    }
}
