#[cfg(test)]
mod tests {
    use crate::api;
    use crate::engine::mcp::McpManager;
    use crate::engine::{remediation, Engine, ProposalExecutionError};
    use actix_web::{http::StatusCode, test, web, App};
    use std::sync::Arc;

    fn set_stacks_block_height(engine: &Engine, height: u64) {
        let mut statuses = engine.service_statuses.write().unwrap();
        let stacks = statuses
            .get_mut("stacks")
            .expect("stacks service status should exist");
        stacks
            .metadata
            .insert("block_height".to_string(), height.to_string());
    }

    fn invalid_testnet_flag() -> bool {
        remediation::is_production_mainnet()
    }

    #[tokio::test]
    async fn test_proposal_lifecycle() {
        let engine = Engine::new();

        // 1. Create a proposal via process_external_settlement
        let payload = serde_json::json!({"testnet": true, "amount": 100});
        let proposal = engine.process_external_settlement("ISO20022", payload);
        let proposal_id = proposal.proposal_id.clone();

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

        // 4. Execution should fail until timelock expires
        let execution_err = engine.execute_proposal(&proposal_id).unwrap_err();
        assert!(matches!(
            execution_err,
            ProposalExecutionError::TimelockNotExpired { .. }
        ));

        let proposals = engine.get_proposals();
        let still_approved = proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap();
        assert_eq!(still_approved.status, "Approved");

        // 5. Advance block height to timelock end and execute successfully
        set_stacks_block_height(&engine, proposal.timelock_end_block);
        engine.execute_proposal(&proposal_id).unwrap();

        let proposals = engine.get_proposals();
        let executed_prop = proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap();
        assert_eq!(executed_prop.status, "Executed");
    }

    #[tokio::test]
    async fn test_settlement_proposal_approve_requires_validation_guard() {
        let engine = web::Data::new(Engine::new());
        let proposal = engine.process_external_settlement("ISO20022", serde_json::json!({}));

        let app =
            test::init_service(App::new().app_data(engine.clone()).configure(api::config)).await;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/api/v1/settlement/proposals/{}/approve?testnet={}",
                proposal.proposal_id,
                invalid_testnet_flag()
            ))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let proposal_after = engine
            .get_proposals()
            .into_iter()
            .find(|p| p.proposal_id == proposal.proposal_id)
            .unwrap();
        assert_eq!(proposal_after.status, "Pending");
    }

    #[tokio::test]
    async fn test_settlement_proposal_execute_requires_validation_guard() {
        let engine = web::Data::new(Engine::new());
        let proposal = engine.process_external_settlement("ISO20022", serde_json::json!({}));
        assert!(engine.approve_proposal(&proposal.proposal_id));
        set_stacks_block_height(engine.get_ref(), proposal.timelock_end_block);

        let app =
            test::init_service(App::new().app_data(engine.clone()).configure(api::config)).await;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/api/v1/settlement/proposals/{}/execute?testnet={}",
                proposal.proposal_id,
                invalid_testnet_flag()
            ))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let proposal_after = engine
            .get_proposals()
            .into_iter()
            .find(|p| p.proposal_id == proposal.proposal_id)
            .unwrap();
        assert_eq!(proposal_after.status, "Approved");
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

        // 3. Advance stack block height past timelock and execute via MCP
        let timelock_end_block = engine
            .get_proposals()
            .into_iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap()
            .timelock_end_block;
        set_stacks_block_height(engine.as_ref(), timelock_end_block);

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
        engine.initialize();
        let status = engine.get_service_status("stacks");
        assert!(status.metadata.contains_key("version"));
    }

    #[tokio::test]
    async fn test_service_status_coverage() {
        let engine = Engine::new();
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
