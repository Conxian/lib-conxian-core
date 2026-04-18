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
