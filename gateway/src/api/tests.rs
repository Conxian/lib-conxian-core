use std::sync::Arc;

use super::*;
use crate::engine::Engine;
use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;

static ENV_VAR_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const PARTNER_INTAKE_API_KEY_ENV: &str = "PARTNER_INTAKE_API_KEY";
const PARTNER_INTAKE_API_KEY_HEADER: &str = "X-Partner-Intake-Key";
const PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, prev }
    }

    fn remove(key: &'static str) -> Self {
        let prev = std::env::var(key).ok();
        std::env::remove_var(key);
        Self { key, prev }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn intake_create_payload() -> serde_json::Value {
    serde_json::json!({
        "partner_name": "Acme Partners",
        "contact_name": "Alice Ops",
        "contact_email": "alice@acme.example",
        "company_name": "Acme Inc",
        "notes": "Priority inbound lead"
    })
}

#[actix_web::test]
async fn test_health_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_status_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/status").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_compliance_zkml_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/compliance/zkml-verify")
        .set_json(serde_json::json!({"proof": "zkml_proof_123"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_financials_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/financials")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_identity_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/identity/0x1234abcd")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_erp_sync_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/erp/sync")
        .set_json(serde_json::json!({"system": "SAP"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_cjcs_spec_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/spec/cjcs")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_dlc_bond_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/finance/bond/BOND-001")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_state_commit_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/state/commit")
        .set_json(serde_json::json!({"state_root": "0xabc123"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_external_settlement_flow() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;

    // 1. Submit ISO 20022 settlement
    let req = test::TestRequest::post()
        .uri("/api/v1/settlement/iso20022")
        .set_json(serde_json::json!({"msg_id": "ISO-001", "amount": 1000}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let proposal: Value = test::read_body_json(resp).await;
    assert_eq!(proposal["status"], "Pending");
    assert!(proposal["proposal_id"]
        .as_str()
        .unwrap()
        .starts_with("prop-iso20022"));

    // 2. Verify 144-block timelock (Stacks height 841500 + 144)
    assert_eq!(proposal["timelock_end_block"], 841644);
    assert_eq!(proposal["yield_routing"], "5/5/90");
    assert_eq!(proposal["capital_status"], "TransitBond");
    assert_eq!(
        proposal["tee_attestation"],
        "VerifiedByStrongBox-Mainnet-v1.0"
    );

    // 3. List proposals
    let req = test::TestRequest::get()
        .uri("/api/v1/settlement/proposals")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let proposals: Vec<Value> = test::read_body_json(resp).await;
    assert_eq!(proposals.len(), 1);
}

#[actix_web::test]
async fn test_sab_wallets_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/sab/wallets")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let wallets: Vec<Value> = test::read_body_json(resp).await;
    assert!(!wallets.is_empty());
    assert_eq!(
        wallets[0]["address"],
        "SPSZXAKV7DWTDZN2601WR31BM51BD3YTQWE97VRM"
    );
}

#[actix_web::test]
async fn test_partner_intake_create_success() {
    let _env_lock = ENV_VAR_MUTEX.lock().await;
    let _partner_key_guard = EnvVarGuard::set(PARTNER_INTAKE_API_KEY_ENV, "partner-intake-secret");

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/intake/partner")
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .insert_header((PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER, "idem-create-001"))
        .set_json(intake_create_payload())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["idempotent_replay"], false);
    assert_eq!(body["lead"]["status"], "new");
    assert_eq!(body["lead"]["partner_name"], "Acme Partners");
    assert!(body["lead"]["id"].as_str().unwrap().starts_with("lead-"));
}

#[actix_web::test]
async fn test_partner_intake_validation_failure() {
    let _env_lock = ENV_VAR_MUTEX.lock().await;
    let _partner_key_guard = EnvVarGuard::set(PARTNER_INTAKE_API_KEY_ENV, "partner-intake-secret");

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/intake/partner")
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .insert_header((PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER, "idem-create-002"))
        .set_json(serde_json::json!({
            "partner_name": "Acme Partners",
            "contact_name": "Alice Ops"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "validation_failed");
}

#[actix_web::test]
async fn test_partner_intake_auth_failure() {
    let _env_lock = ENV_VAR_MUTEX.lock().await;
    let _partner_key_guard = EnvVarGuard::set(PARTNER_INTAKE_API_KEY_ENV, "partner-intake-secret");

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/intake/partner")
        .insert_header((PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER, "idem-create-003"))
        .set_json(intake_create_payload())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_partner_intake_idempotent_replay() {
    let _env_lock = ENV_VAR_MUTEX.lock().await;
    let _partner_key_guard = EnvVarGuard::set(PARTNER_INTAKE_API_KEY_ENV, "partner-intake-secret");

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;

    let first_req = test::TestRequest::post()
        .uri("/api/v1/intake/partner")
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .insert_header((PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER, "idem-replay-001"))
        .set_json(intake_create_payload())
        .to_request();
    let first_resp = test::call_service(&app, first_req).await;
    assert_eq!(first_resp.status(), StatusCode::CREATED);
    let first_body: Value = test::read_body_json(first_resp).await;

    let replay_req = test::TestRequest::post()
        .uri("/api/v1/intake/partner")
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .insert_header((PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER, "idem-replay-001"))
        .set_json(intake_create_payload())
        .to_request();
    let replay_resp = test::call_service(&app, replay_req).await;
    assert_eq!(replay_resp.status(), StatusCode::OK);
    let replay_body: Value = test::read_body_json(replay_resp).await;

    assert_eq!(replay_body["idempotent_replay"], true);
    assert_eq!(replay_body["lead"]["id"], first_body["lead"]["id"]);
}

#[actix_web::test]
async fn test_partner_intake_valid_and_invalid_transitions() {
    let _env_lock = ENV_VAR_MUTEX.lock().await;
    let _partner_key_guard = EnvVarGuard::set(PARTNER_INTAKE_API_KEY_ENV, "partner-intake-secret");

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;

    let create_req = test::TestRequest::post()
        .uri("/api/v1/intake/partner")
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .insert_header((PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER, "idem-transition-001"))
        .set_json(intake_create_payload())
        .to_request();
    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let created: Value = test::read_body_json(create_resp).await;
    let lead_id = created["lead"]["id"].as_str().unwrap();

    let assigned_req = test::TestRequest::post()
        .uri(&format!("/api/v1/intake/partner/{lead_id}/status"))
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .set_json(serde_json::json!({"status": "assigned", "owner": "ops-1"}))
        .to_request();
    let assigned_resp = test::call_service(&app, assigned_req).await;
    assert_eq!(assigned_resp.status(), StatusCode::OK);
    let assigned: Value = test::read_body_json(assigned_resp).await;
    assert_eq!(assigned["status"], "assigned");
    assert_eq!(assigned["owner"], "ops-1");

    let in_progress_req = test::TestRequest::post()
        .uri(&format!("/api/v1/intake/partner/{lead_id}/status"))
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .set_json(serde_json::json!({"status": "in_progress"}))
        .to_request();
    let in_progress_resp = test::call_service(&app, in_progress_req).await;
    assert_eq!(in_progress_resp.status(), StatusCode::OK);

    let invalid_req = test::TestRequest::post()
        .uri(&format!("/api/v1/intake/partner/{lead_id}/status"))
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .set_json(serde_json::json!({"status": "closed"}))
        .to_request();
    let invalid_resp = test::call_service(&app, invalid_req).await;
    assert_eq!(invalid_resp.status(), StatusCode::CONFLICT);

    let escalated_req = test::TestRequest::post()
        .uri(&format!("/api/v1/intake/partner/{lead_id}/status"))
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .set_json(
            serde_json::json!({"status": "escalated", "escalation_reason": "needs legal review", "escalated_to": "legal"}),
        )
        .to_request();
    let escalated_resp = test::call_service(&app, escalated_req).await;
    assert_eq!(escalated_resp.status(), StatusCode::OK);

    let closed_req = test::TestRequest::post()
        .uri(&format!("/api/v1/intake/partner/{lead_id}/status"))
        .insert_header((PARTNER_INTAKE_API_KEY_HEADER, "partner-intake-secret"))
        .set_json(serde_json::json!({"status": "closed"}))
        .to_request();
    let closed_resp = test::call_service(&app, closed_req).await;
    assert_eq!(closed_resp.status(), StatusCode::OK);
    let closed: Value = test::read_body_json(closed_resp).await;
    assert_eq!(closed["status"], "closed");
    assert!(closed["closed_at"].is_string());
}

#[actix_web::test]
async fn test_bitvm2_verify_state_root_missing_vk() {
    let _env_lock = ENV_VAR_MUTEX.lock().await;
    let key = lib_conxian_core::bitvm2::ENV_BITVM2_GROTH16_VK_B64;
    let _vk_guard = EnvVarGuard::remove(key);

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/bitvm2/verify-state-root")
        .set_json(serde_json::json!({"state_root": "0x0000000000000000000000000000000000000000000000000000000000000000", "proof": ""}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    );
}

#[actix_web::test]
async fn test_mcp_tools_list() {
    let engine = Arc::new(Engine::new());
    engine.initialize();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(engine))
            .configure(crate::api::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/mcp")
        .set_json(serde_json::json!({
            "method": "tools/list",
            "params": {}
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["tools"].is_array());
    assert!(body["tools"].as_array().unwrap().len() >= 4);
}

#[actix_web::test]
async fn test_mcp_draft_intent() {
    let engine = Arc::new(Engine::new());
    engine.initialize();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(engine))
            .configure(crate::api::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/mcp")
        .set_json(serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "draft_financial_intent",
                "arguments": {
                    "type": "YieldOptimization",
                    "details": { "target": "sBTC-LP" }
                }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["requires_handshake"].as_bool().unwrap());
    assert!(body["proposal_id"].is_string());
}

#[actix_web::test]
async fn test_bitvm2_segments_endpoint() {
    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/bitvm2/segments/0xabc")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["segments"].is_array());
    assert_eq!(body["segments"].as_array().unwrap().len(), 364);
}
