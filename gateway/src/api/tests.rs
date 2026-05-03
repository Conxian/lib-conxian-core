#[cfg(test)]
mod tests {
    use crate::engine::mcp::McpManager;
    use crate::engine::Engine;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_proposal_lifecycle() {
        let engine = Engine::new();
        std::env::set_var("CONXIAN_NETWORK", "testnet");

        // 1. Create a proposal via process_external_settlement
        let payload = serde_json::json!({"testnet": true, "amount": 100});
        let proposal = engine.process_external_settlement("ISO20022", payload);
        let proposal_id = proposal.proposal_id;

        assert_eq!(proposal.status, "Pending");

        // 2. Verify it exists in the engine
        let proposals = engine.get_proposals();
        assert!(proposals.iter().any(|p| p.proposal_id == proposal_id));

        // 3. Approve it
        let approved = engine.approve_proposal(&proposal_id);
        assert!(approved);

        let proposals = engine.get_proposals();
        let approved_prop = proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap();
        assert_eq!(approved_prop.status, "Approved");

        // 4. Execute it
        let executed = engine.execute_proposal(&proposal_id);
        assert!(executed);

        let proposals = engine.get_proposals();
        let executed_prop = proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap();
        assert_eq!(executed_prop.status, "Executed");
    }

    #[tokio::test]
    async fn test_mcp_list_proposals() {
        let engine = Arc::new(Engine::new());
        let manager = McpManager::new(Arc::clone(&engine));

        // Create a proposal
        engine.process_external_settlement("BRICS", serde_json::json!({"testnet": true}));

        let result = manager
            .handle_call("list_state_proposals", serde_json::json!({}))
            .await;
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.contains("brics"));
        assert!(text.contains("Pending"));
    }

    #[tokio::test]
    async fn test_mcp_proposal_approval_and_execution() {
        let engine = Arc::new(Engine::new());
        let manager = McpManager::new(Arc::clone(&engine));

        // 1. Draft an intent via MCP
        let result = manager
            .handle_call(
                "draft_financial_intent",
                serde_json::json!({
                    "type": "CrossChainManeuver",
                    "details": {"from": "Bitcoin", "to": "Stacks", "amount": 0.5}
                }),
            )
            .await;

        let proposal_id = result["proposal_id"].as_str().unwrap().to_string();

        // 2. Approve via MCP
        let approve_result = manager
            .handle_call(
                "approve_state_proposal",
                serde_json::json!({
                    "proposal_id": proposal_id
                }),
            )
            .await;

        assert!(approve_result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("approved successfully"));

        // 3. Execute via MCP
        let execute_result = manager
            .handle_call(
                "execute_state_proposal",
                serde_json::json!({
                    "proposal_id": proposal_id
                }),
            )
            .await;

        assert!(execute_result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("executed successfully"));

        // 4. Verify status in engine
        let proposals = engine.get_proposals();
        let prop = proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap();
        assert_eq!(prop.status, "Executed");
    }

    #[tokio::test]
    async fn test_rpc_metadata_updates() {
        let engine = Engine::new();
        std::env::set_var("CONXIAN_NETWORK", "testnet");
        engine.initialize();
        let status = engine.get_service_status("stacks");
        assert!(status.metadata.contains_key("version"));
        let bitvm_status = engine.get_service_status("bitvm2");
        assert!(bitvm_status.metadata.contains_key("bitvm_challenge_status"));
    }

    #[tokio::test]
    async fn test_service_status_coverage() {
        let engine = Engine::new();
        std::env::set_var("CONXIAN_NETWORK", "testnet");
        engine.initialize();
        let layers = vec![
            "bitvm2", "bob", "merlin", "botanix", "hemi", "alpen", "bison",
        ];

        for layer in layers {
            let status = engine.get_service_status(layer);
            assert_eq!(status.name, layer);
        }
    }
}
