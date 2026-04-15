use crate::engine::Engine;
use crate::engine::StateProposal;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

pub struct McpManager {
    engine: Arc<Engine>,
}

impl McpManager {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub fn get_telemetry_tool(&self) -> serde_json::Value {
        json!({
            "name": "get_system_telemetry",
            "description": "Retrieve real-time telemetry for the Conxian Gateway, including TVL and node status.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        })
    }

    pub fn get_proof_tool(&self) -> serde_json::Value {
        json!({
            "name": "get_protocol_proof",
            "description": "Retrieve a specific protocol proof (BitVM, Citrea, etc.) for auditing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "protocol": { "type": "string", "enum": ["bitvm", "citrea"] },
                    "id": { "type": "string" }
                },
                "required": ["protocol", "id"]
            }
        })
    }

    pub fn get_yield_oracle_tool(&self) -> serde_json::Value {
        json!({
            "name": "get_yield_metrics",
            "description": "Retrieve read-only metrics regarding native bond financing and yield streams.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "asset": { "type": "string", "default": "sBTC" }
                }
            }
        })
    }

    pub fn get_industrial_intents_tool(&self) -> serde_json::Value {
        json!({
            "name": "list_industrial_intents",
            "description": "Broadcast self-describing tool schemas for FSOC validation or settlement triggers.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        })
    }

    pub fn get_draft_intent_tool(&self) -> serde_json::Value {
        json!({
            "name": "draft_financial_intent",
            "description": "Construct a complex financial intent for human signing. Subject to 144-block timelock.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "type": { "type": "string", "enum": ["CrossChainManeuver", "YieldOptimization"] },
                    "details": { "type": "object" }
                },
                "required": ["type", "details"]
            }
        })
    }

    pub async fn handle_call(
        &self,
        tool_name: &str,
        _arguments: serde_json::Value,
    ) -> serde_json::Value {
        match tool_name {
            "get_system_telemetry" => {
                let status = self.engine.get_status();
                json!({ "content": [{ "type": "text", "text": format!("System Status: {}", status) }] })
            }
            "get_protocol_proof" => {
                let protocol = _arguments
                    .get("protocol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let id = _arguments.get("id").and_then(|v| v.as_str()).unwrap_or("");
                json!({ "content": [{ "type": "text", "text": format!("Proof data for {} (ID: {}) retrieved.", protocol, id) }] })
            }
            "get_yield_metrics" => {
                let financials = self.engine.get_financial_metrics();
                json!({ "content": [{ "type": "text", "text": format!("Financial Metrics: {:?}", financials) }] })
            }
            "list_industrial_intents" => {
                json!({
                    "content": [{
                        "type": "text",
                        "text": "Industrial Intents available: SettlementProposal, FSOCValidation, AssetPegIn, YieldRebalance"
                    }],
                    "schemas": [
                        { "intent": "SettlementProposal", "fields": ["protocol", "payload", "tee_attestation"] },
                        { "intent": "FSOCValidation", "fields": ["transaction_hash", "proof_type"] }
                    ]
                })
            }
            "draft_financial_intent" => {
                let intent_type = _arguments
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let _details = _arguments.get("details").cloned().unwrap_or(json!({}));

                let trigger_id = format!("agent-intent-{}", Utc::now().timestamp());
                let proposal_id = format!("prop-{}", trigger_id);

                let proposal = StateProposal {
                    proposal_id: proposal_id.clone(),
                    trigger_id,
                    proposed_state: format!("AgentDraft:{}", intent_type),
                    timelock_end_block: 841500 + 144, // Default + 144
                    status: "Pending".to_string(),
                    tee_attestation: "DraftedByAgent-v1.0".to_string(),
                    yield_routing: "PendingApproval".to_string(),
                    capital_status: "TransitBond".to_string(),
                };

                self.engine
                    .state_proposals
                    .write()
                    .unwrap()
                    .insert(proposal_id.clone(), proposal);

                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Drafted financial intent ({}). Created StateProposal: {}. This intent requires human signing and is subject to a 144-block timelock.", intent_type, proposal_id)
                    }],
                    "proposal_id": proposal_id,
                    "requires_handshake": true
                })
            }
            _ => {
                json!({ "isError": true, "content": [{ "type": "text", "text": "Tool not found" }] })
            }
        }
    }
}
