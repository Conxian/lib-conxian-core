import sys

content = open('gateway/src/engine/mod.rs').read()

old_bitvm_proof = """    pub fn get_bitvm_proof(&self, proof_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "proof_id": proof_id,
            "status": "verified",
            "computation_type": "sha256_hash_check",
            "challenge_window_blocks": 100,
            "operator_deposit_btc": 1.5
        })
    }"""

new_bitvm_proof = """    pub fn get_bitvm_proof(&self, proof_id: &str) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "proof_id": proof_id,
                "status": "ConnectionRequired",
                "error": "Mainnet node connection required for real-time proof auditing.",
                "remediation": "Configure BITCOIN_RPC_URL and BITVM_DISPROVER_ENDPOINT"
            });
        }
        serde_json::json!({
            "proof_id": proof_id,
            "status": "verified",
            "computation_type": "sha256_hash_check",
            "challenge_window_blocks": 100,
            "operator_deposit_btc": 1.5
        })
    }"""

old_citrea_proof = """    pub fn get_citrea_proof(&self, batch_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "batch_id": batch_id,
            "status": "Finalized",
            "zk_proof": "0xabc...",
            "settlement_tx": "0x123...",
            "timestamp": Utc::now()
        })
    }"""

new_citrea_proof = """    pub fn get_citrea_proof(&self, batch_id: &str) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "batch_id": batch_id,
                "status": "ConnectionRequired",
                "error": "Mainnet node connection required for Citrea ZK-proof verification.",
                "remediation": "Configure CITREA_RPC_URL"
            });
        }
        serde_json::json!({
            "batch_id": batch_id,
            "status": "Finalized",
            "zk_proof": "0xabc...",
            "settlement_tx": "0x123...",
            "timestamp": Utc::now()
        })
    }"""

content = content.replace(old_bitvm_proof, new_bitvm_proof)
content = content.replace(old_citrea_proof, new_citrea_proof)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
