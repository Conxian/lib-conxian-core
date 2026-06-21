use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeploymentPlan {
    pub project_id: String,
    pub version: String,
    pub environment: String,
    pub contracts: Vec<ContractDeployment>,
    pub parameters: HashMap<String, String>,
    pub nakamoto_integrity_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractDeployment {
    pub name: String,
    pub source_hash: String,
    pub compiler_version: String,
    pub traits: Vec<String>,
}

impl DeploymentPlan {
    pub fn new(project_id: &str, version: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            version: version.to_string(),
            environment: "mainnet".to_string(),
            contracts: vec![],
            parameters: HashMap::new(),
            nakamoto_integrity_hash: "sha256:pending".to_string(),
        }
    }

    pub fn add_contract(&mut self, name: &str, hash: &str) {
        self.contracts.push(ContractDeployment {
            name: name.to_string(),
            source_hash: hash.to_string(),
            compiler_version: "2.5".to_string(),
            traits: vec!["sip-010".to_string()],
        });
    }

    pub fn to_agent_readable(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

// --- CON-1237: Shared Artifact Schemas ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DeploymentStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

/// A machine-readable record of a completed or failed deployment.
/// Used by Platform, Gateway, and Nexus to track ecosystem state.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeploymentManifest {
    pub manifest_version: String,
    pub project_id: String,
    pub deployment_id: String,
    pub environment: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: DeploymentStatus,
    pub contracts: Vec<ContractDeploymentRecord>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractDeploymentRecord {
    pub contract_name: String,
    pub contract_address: String,
    pub tx_id: String,
    pub block_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum VerificationOutcome {
    Pending,
    Pass,
    Fail,
    Warning,
}

/// The result of a post-deployment verification check.
/// Provides the evidence required for high-confidence mainnet activation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationResult {
    pub verification_id: String,
    pub deployment_id: String,
    pub outcome: VerificationOutcome,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evidence: Vec<VerificationEvidence>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationEvidence {
    pub component: String,
    pub check_type: String,
    pub proof_hash: Option<String>,
    pub details: HashMap<String, String>,
}

impl DeploymentManifest {
    pub fn new(project_id: &str, deployment_id: &str, environment: &str) -> Self {
        Self {
            manifest_version: "1.0.0".to_string(),
            project_id: project_id.to_string(),
            deployment_id: deployment_id.to_string(),
            environment: environment.to_string(),
            timestamp: chrono::Utc::now(),
            status: DeploymentStatus::Pending,
            contracts: vec![],
            metadata: HashMap::new(),
        }
    }

    pub fn add_contract(&mut self, name: &str, address: &str, tx_id: &str) {
        self.contracts.push(ContractDeploymentRecord {
            contract_name: name.to_string(),
            contract_address: address.to_string(),
            tx_id: tx_id.to_string(),
            block_height: None,
        });
    }
}

impl VerificationResult {
    pub fn new(verification_id: &str, deployment_id: &str) -> Self {
        Self {
            verification_id: verification_id.to_string(),
            deployment_id: deployment_id.to_string(),
            outcome: VerificationOutcome::Pending,
            timestamp: chrono::Utc::now(),
            evidence: vec![],
            errors: vec![],
        }
    }

    pub fn add_evidence(&mut self, component: &str, check_type: &str, hash: Option<String>) {
        self.evidence.push(VerificationEvidence {
            component: component.to_string(),
            check_type: check_type.to_string(),
            proof_hash: hash,
            details: HashMap::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_manifest_serialization() {
        let mut manifest = DeploymentManifest::new("proj-123", "deploy-456", "production");
        manifest.status = DeploymentStatus::Completed;
        manifest.add_contract("vault-core", "SP123...ABC", "0xabc...def");
        manifest
            .metadata
            .insert("triggered_by".to_string(), "jules-agent".to_string());

        let json = serde_json::to_string(&manifest).unwrap();
        let decoded: DeploymentManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.project_id, "proj-123");
        assert_eq!(decoded.status, DeploymentStatus::Completed);
        assert_eq!(decoded.contracts.len(), 1);
        assert_eq!(decoded.contracts[0].contract_name, "vault-core");
        assert_eq!(decoded.metadata.get("triggered_by").unwrap(), "jules-agent");
    }

    #[test]
    fn test_verification_result_serialization() {
        let mut result = VerificationResult::new("verify-789", "deploy-456");
        assert_eq!(result.outcome, VerificationOutcome::Pending);
        result.outcome = VerificationOutcome::Pass;
        result.add_evidence(
            "nexus-zkvm",
            "state-proof",
            Some("sha256:proof123".to_string()),
        );
        result.errors.push("None".to_string());

        let json = serde_json::to_string(&result).unwrap();
        let decoded: VerificationResult = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.verification_id, "verify-789");
        assert_eq!(decoded.outcome, VerificationOutcome::Pass);
        assert_eq!(decoded.evidence.len(), 1);
        assert_eq!(decoded.evidence[0].component, "nexus-zkvm");
        assert_eq!(decoded.errors.len(), 1);
    }
}
