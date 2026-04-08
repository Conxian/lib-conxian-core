use std::sync::Arc;
use super::*;
use crate::engine::Engine;
use actix_web::{test, web, App};
use serde_json::Value;

#[actix_web::test]
async fn test_health_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_status_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/status").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_compliance_zkml_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
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
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/financials")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_identity_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/identity/0x1234abcd")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_erp_sync_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
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
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/spec/cjcs")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_dlc_bond_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/finance/bond/BOND-001")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_state_commit_endpoint() {
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
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
    let engine_arc = Arc::new(Engine::new()); engine_arc.initialize(); let engine = web::Data::from(engine_arc);
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
    assert_eq!(proposal["tee_attestation"], "VerifiedByStrongBox-Mainnet-v1.0");


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
    let req = test::TestRequest::get().uri("/api/v1/sab/wallets").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let wallets: Vec<Value> = test::read_body_json(resp).await;
    assert!(!wallets.is_empty());
    assert_eq!(wallets[0]["address"], "SPSZXAKV7DWTDZN2601WR31BM51BD3YTQWE97VRM");
}

#[actix_web::test]
async fn test_bitvm2_verify_state_root_missing_vk() {
    struct EnvVarGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    let key = lib_conxian_core::bitvm2::ENV_BITVM2_GROTH16_VK_B64;
    let prev = std::env::var(key).ok();
    std::env::remove_var(key);
    let _guard = EnvVarGuard { key, prev };

    let engine_arc = Arc::new(Engine::new());
    engine_arc.initialize();
    let engine = web::Data::from(engine_arc);
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/bitvm2/verify-state-root")
        .set_json(serde_json::json!({"state_root": "0x0000000000000000000000000000000000000000000000000000000000000000", "proof": ""}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::SERVICE_UNAVAILABLE);
}
