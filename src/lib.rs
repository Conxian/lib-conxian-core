pub mod bitvm2;
pub mod deployment;
pub mod musig2;

pub mod cjcs;
pub mod contract_bridge;
pub mod gateway;
pub mod wallet;

pub use contract_bridge::{ClarityCall, ContractBridge, SignedContractCall};
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
