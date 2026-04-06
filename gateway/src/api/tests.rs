use super::*;
use crate::engine::Engine;
use actix_web::{test, web, App};
use serde_json::Value;

#[actix_web::test]
async fn test_health_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_status_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/status").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_compliance_zkml_endpoint() {
    let engine = web::Data::new(Engine::new());
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
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/financials")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_identity_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/identity/0x1234abcd")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_erp_sync_endpoint() {
    let engine = web::Data::new(Engine::new());
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
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/spec/cjcs")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_dlc_bond_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/finance/bond/BOND-001")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_state_commit_endpoint() {
    let engine = web::Data::new(Engine::new());
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
    let engine = web::Data::new(Engine::new());
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

    // 2. Verify 144-block timelock (Stacks height 841000 + 144)
    assert_eq!(proposal["timelock_end_block"], 841144);

    // 3. List proposals
    let req = test::TestRequest::get()
        .uri("/api/v1/settlement/proposals")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let proposals: Vec<Value> = test::read_body_json(resp).await;
    assert_eq!(proposals.len(), 1);
}
