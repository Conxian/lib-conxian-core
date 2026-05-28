#[cfg(test)]
mod tests {
    use crate::engine::anchoring::AnchoringError;
    use crate::engine::mcp::McpManager;
    use crate::engine::{BitcoinTxLifecycleEvent, BitcoinTxTransitionError, Engine};
    use actix_web::http::StatusCode;
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
        assert_eq!(status.rail_metadata.rail_family, "anchored_l2");
        assert!(!status
            .rail_metadata
            .trust_assumptions
            .security_anchor
            .is_empty());
        let bitvm_status = engine.get_service_status("bitvm2");
        assert!(bitvm_status.metadata.contains_key("bitvm_challenge_status"));
        assert_eq!(bitvm_status.rail_metadata.rail_family, "optimistic_rollup");
        assert!(!bitvm_status
            .rail_metadata
            .operational_capabilities
            .supported_flows
            .is_empty());
    }

    #[tokio::test]
    async fn test_service_status_coverage() {
        let engine = Engine::new();
        std::env::set_var("CONXIAN_NETWORK", "testnet");
        engine.initialize();
        let statuses = engine.get_all_service_statuses();

        assert!(!statuses.is_empty());

        for status in statuses {
            assert!(!status.rail_metadata.rail_family.is_empty());
            assert!(!status
                .rail_metadata
                .trust_assumptions
                .operator_dependency
                .is_empty());
            assert!(!status
                .rail_metadata
                .finality_semantics
                .confirmation_model
                .is_empty());
            assert!(!status
                .rail_metadata
                .custody_model
                .asset_control_model
                .is_empty());
            assert!(!status
                .rail_metadata
                .compliance_constraints
                .baseline_controls
                .is_empty());
            assert!(!status
                .rail_metadata
                .operational_capabilities
                .supported_flows
                .is_empty());
        }
    }

    #[tokio::test]
    async fn test_unknown_service_metadata_is_explicit() {
        let engine = Engine::new();
        let status = engine.get_service_status("unregistered-rail");

        assert_eq!(status.status, "unknown");
        assert_eq!(status.rail_metadata.rail_family, "unknown");
        assert_eq!(
            status
                .rail_metadata
                .operational_capabilities
                .supported_flows,
            vec!["status_visibility_only".to_string()]
        );
    }

    #[test]
    fn test_anchoring_error_mapping_retryable_adapter_failure() {
        let response = super::super::anchoring_error_response(AnchoringError::AdapterFailure {
            adapter: "on_chain".to_string(),
            code: "rpc_unavailable".to_string(),
            message: "upstream node unavailable".to_string(),
            retryable: true,
        });

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_anchoring_error_mapping_non_retryable_adapter_failure() {
        let response = super::super::anchoring_error_response(AnchoringError::AdapterFailure {
            adapter: "tableland".to_string(),
            code: "schema_mismatch".to_string(),
            message: "payload rejected".to_string(),
            retryable: false,
        });

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_bitcoin_transition_error_mapping_feature_disabled() {
        let response =
            super::super::map_bitcoin_transition_error(BitcoinTxTransitionError::FeatureDisabled);
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_bitcoin_transition_error_mapping_invalid_transition() {
        let response = super::super::map_bitcoin_transition_error(
            BitcoinTxTransitionError::InvalidTransition {
                from: crate::engine::BitcoinTxLifecycleState::Draft,
                event: BitcoinTxLifecycleEvent::QueueBroadcast,
                reason: "queue_broadcast requires signed or reorged state".to_string(),
            },
        );

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_bitcoin_transition_request_validation_requires_tx_id_and_event() {
        let validation = super::super::BitcoinTxTransitionRequest {
            tx_id: None,
            event: None,
            idempotency_key: None,
            rationale: None,
            fee_rate_sat_vb: None,
            attempt: None,
            confirmations_observed: None,
            required_confirmations: None,
            reorg_depth: None,
            dead_letter_reason: None,
        }
        .validate();

        assert!(validation.is_err());
        let errors = validation.expect_err("validation should fail");
        assert!(errors.iter().any(|err| err == "tx_id is required"));
        assert!(errors.iter().any(|err| err == "event is required"));
    }

    #[test]
    fn test_bitcoin_transition_request_validation_builds_engine_input() {
        let validation = super::super::BitcoinTxTransitionRequest {
            tx_id: Some(" tx-abc ".to_string()),
            event: Some(BitcoinTxLifecycleEvent::ConfirmationsObserved),
            idempotency_key: Some(" req-1 ".to_string()),
            rationale: Some(" lifecycle update ".to_string()),
            fee_rate_sat_vb: Some(14),
            attempt: Some(2),
            confirmations_observed: Some(3),
            required_confirmations: Some(6),
            reorg_depth: None,
            dead_letter_reason: Some(" confirmation update ".to_string()),
        }
        .validate()
        .expect("validation should succeed");

        assert_eq!(validation.tx_id, "tx-abc");
        assert_eq!(
            validation.event,
            BitcoinTxLifecycleEvent::ConfirmationsObserved
        );
        assert_eq!(validation.idempotency_key, Some("req-1".to_string()));
        assert_eq!(validation.rationale, Some("lifecycle update".to_string()));
        assert_eq!(validation.fee_rate_sat_vb, Some(14));
        assert_eq!(validation.attempt, Some(2));
        assert_eq!(validation.confirmations_observed, Some(3));
        assert_eq!(validation.required_confirmations, Some(6));
        assert_eq!(
            validation.dead_letter_reason,
            Some("confirmation update".to_string())
        );
    }
}
