//! # lib-conxian-core / Vault SDK
//!
//! This library provides the production-grade primitives for native Bitcoin applications.
//! It is divided into two primary areas:
//!
//! 1. **Vault SDK (Public Boundary)**: The `sdk_primitive` and `wallet` modules provide
//!    hardware-backed signing and policy enforcement for third-party integrators.
//! 2. **Protocol Primitives (Internal Core)**: Modules like `musig2`, `bitvm2`, and chain adapters
//!    provide the low-level logic required for Bitcoin-anchored orchestration.

pub mod adapters;
pub mod anchoring;
pub mod babylon;
pub mod bitvm2;
pub mod cjcs;
pub mod contract_bridge;
pub mod control_model;
pub mod deployment;
pub mod fedimint;
pub mod protocol;
pub mod musig2;
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
pub use sdk_primitive::{SigningPolicy, VaultSDK};
pub use wallet::{sign_transaction, Wallet};

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
