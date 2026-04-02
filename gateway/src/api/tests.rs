use crate::api::config;
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
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "operational");
}

#[actix_web::test]
async fn test_stacks_handler() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/stacks").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_lightning_handler() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/lightning")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_liquid_handler() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get().uri("/api/v1/liquid").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_rootstock_handler() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/rootstock")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_compliance_zkml_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/compliance/zkml-verify")
        .set_json(serde_json::json!({"proof": "zkml_valid_proof"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["verified"], true);
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
    let body: Value = test::read_body_json(resp).await;
    assert!(body["mrr_usd"].as_f64().unwrap() > 0.0);
}

#[actix_web::test]
async fn test_identity_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/identity/0x1234567890abcdef_verified")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert!(body["ens_name"].as_str().unwrap().contains(".eth"));
}

#[actix_web::test]
async fn test_erp_sync_endpoint() {
    let engine = web::Data::new(Engine::new());
    let app = test::init_service(App::new().app_data(engine).configure(config)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/erp/sync")
        .set_json(serde_json::json!({"system": "Oracle"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["erp_system"], "Oracle");
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
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["version"], "2.0.0");
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
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["persistence"], "Decentralized (Tableland)");
}
