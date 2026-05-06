#[cfg(test)]
mod integration_tests {
    use crate::api;
    use crate::engine::mcp::McpManager;
    use crate::engine::{remediation, Engine, ProposalExecutionError};
    use actix_web::{http::StatusCode, test, web, App};
    use std::sync::Arc;

    fn ensure_stacks_service(engine: &Engine) {
        let mut statuses = engine.service_statuses.write().unwrap();
        if !statuses.contains_key("stacks") {
            statuses.insert(
                "stacks".to_string(),
                crate::engine::ServiceStatus {
                    name: "stacks".to_string(),
                    status: "active".to_string(),
                    last_checked: chrono::Utc::now(),
                    latency_ms: 0,
                    trust_model: "PoX".to_string(),
                    risk_level: "Low".to_string(),
                    risk_assessment: None,
                    data_availability: "On-chain".to_string(),
                    settlement: "Bitcoin".to_string(),
                    bridge_security: "sBTC".to_string(),
                    tvl_usd: 0.0,
                    version: None,
                    metadata: std::collections::HashMap::new(),
                },
            );
        }
    }

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

    fn setup_test_env() {
        std::env::set_var("GATEWAY_ADMIN_API_KEY", "secret-admin-key");
        std::env::set_var("CONXIAN_NETWORK", "testnet");
    }

    #[tokio::test]
    async fn test_proposal_lifecycle() {
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);

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
        setup_test_env();
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);
        let engine_data = web::Data::new(engine);
        let proposal = engine_data
            .process_external_settlement("ISO20022", serde_json::json!({"testnet": true}));

        let app = test::init_service(
            App::new()
                .app_data(engine_data.clone())
                .configure(api::config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/api/v1/settlement/proposals/{}/approve?testnet={}",
                proposal.proposal_id,
                invalid_testnet_flag()
            ))
            .insert_header(("X-Gateway-Admin-Key", "secret-admin-key"))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let proposal_after = engine_data
            .get_proposals()
            .into_iter()
            .find(|p| p.proposal_id == proposal.proposal_id)
            .unwrap();
        assert_eq!(proposal_after.status, "Pending");
    }

    #[tokio::test]
    async fn test_settlement_proposal_execute_requires_validation_guard() {
        setup_test_env();
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);
        let engine_data = web::Data::new(engine);
        let proposal = engine_data
            .process_external_settlement("ISO20022", serde_json::json!({"testnet": true}));
        assert!(engine_data.approve_proposal(&proposal.proposal_id));
        set_stacks_block_height(engine_data.get_ref(), proposal.timelock_end_block);

        let app = test::init_service(
            App::new()
                .app_data(engine_data.clone())
                .configure(api::config),
        )
        .await;

        let req = test::TestRequest::post()
            .uri(&format!(
                "/api/v1/settlement/proposals/{}/execute?testnet={}",
                proposal.proposal_id,
                invalid_testnet_flag()
            ))
            .insert_header(("X-Gateway-Admin-Key", "secret-admin-key"))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let proposal_after = engine_data
            .get_proposals()
            .into_iter()
            .find(|p| p.proposal_id == proposal.proposal_id)
            .unwrap();
        assert_eq!(proposal_after.status, "Approved");
    }

    #[tokio::test]
    async fn test_mcp_list_proposals() {
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);
        let engine_arc = Arc::new(engine);
        let manager = McpManager::new(Arc::clone(&engine_arc));

        // Create a proposal
        engine_arc.process_external_settlement("BRICS", serde_json::json!({"testnet": true}));

        let result = manager
            .handle_call("list_state_proposals", serde_json::json!({}))
            .await;
        let text = result["content"][0]["text"].as_str().unwrap();

        assert!(text.to_lowercase().contains("brics"));
        assert!(text.contains("Pending"));
    }

    #[tokio::test]
    async fn test_mcp_proposal_approval_and_execution() {
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);
        let engine_arc = Arc::new(engine);
        let manager = McpManager::new(Arc::clone(&engine_arc));

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
        let timelock_end_block = engine_arc
            .get_proposals()
            .into_iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap()
            .timelock_end_block;
        set_stacks_block_height(engine_arc.as_ref(), timelock_end_block);

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
        let proposals = engine_arc.get_proposals();
        let prop = proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .unwrap();
        assert_eq!(prop.status, "Executed");
    }

    #[tokio::test]
    async fn test_rpc_metadata_updates() {
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);
        engine.initialize();
        let status = engine.get_service_status("stacks");
        assert!(status.metadata.contains_key("version"));
        let bitvm_status = engine.get_service_status("bitvm2");
        assert!(bitvm_status.metadata.contains_key("bitvm_challenge_status"));
    }

    #[tokio::test]
    async fn test_service_status_coverage() {
        setup_test_env();
        let engine = Engine::new();
        ensure_stacks_service(&engine);
        engine.initialize();
        let layers = vec![
            "bitvm2", "bob", "merlin", "botanix", "hemi", "alpen", "bison",
        ];

        for layer in layers {
            let status = engine.get_service_status(layer);
            assert_eq!(status.name, layer);
        }
    }

    #[tokio::test]
    async fn test_admin_auth_required_for_sensitive_endpoints() {
        setup_test_env();
        let engine = Arc::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(Arc::clone(&engine)))
                .configure(api::config),
        )
        .await;

        // 1. Test Settlement Approval with NO key (Expect 401 or 503 if env not set)
        // Here we override env in the same process, so we must be careful with concurrency.
        // But cargo test runs tests in separate threads, and std::env is process-wide.
        // To be safe, we test the rejection logic.

        std::env::remove_var("GATEWAY_ADMIN_API_KEY");
        let req = test::TestRequest::post()
            .uri("/api/v1/settlement/proposals/prop-1/approve")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

        std::env::set_var("GATEWAY_ADMIN_API_KEY", "secret-admin-key");
        let req = test::TestRequest::post()
            .uri("/api/v1/settlement/proposals/prop-1/approve")
            .insert_header(("X-Gateway-Admin-Key", "wrong-key"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
