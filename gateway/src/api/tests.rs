#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use crate::api::config;
    use crate::engine::Engine;
    use serde_json::Value;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
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
    async fn test_bisq_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bisq").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "bisq");
        assert!(body["metadata"]["active_offers"].is_string());
    }

    #[actix_web::test]
    async fn test_stacks_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/stacks").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "stacks");
        assert!(body["metadata"]["block_height"].is_string());
    }

    #[actix_web::test]
    async fn test_reserves_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/reserves").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body.is_array());
        assert!(body[0]["asset"].is_string());
        assert!(body[0]["total_supplied"].is_number());
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("gateway_uptime_seconds"));
    }

    #[actix_web::test]
    async fn test_layers_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/layers").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body.is_object());
        assert!(body["stacks"].is_object());
        assert_eq!(body["stacks"]["name"], "stacks");
    }

    #[actix_web::test]
    async fn test_babylon_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/babylon").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "babylon");
    }

    #[actix_web::test]
    async fn test_lightning_invoice_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/lightning/invoice")
            .set_json(serde_json::json!({"amount_msat": 50000, "description": "Test"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body["invoice"].is_string());
    }

    #[actix_web::test]
    async fn test_stacks_contract_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/stacks/contract/ST123").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["contract_id"], "ST123");
    }

    #[actix_web::test]
    async fn test_merlin_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/merlin").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "merlin");
    }

    #[actix_web::test]
    async fn test_rgb_contract_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/rgb/contract/RGB123").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["contract_id"], "RGB123");
    }

    #[actix_web::test]
    async fn test_citrea_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/citrea").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "citrea");
    }

    #[actix_web::test]
    async fn test_bitlayer_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bitlayer").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "bitlayer");
    }

    #[actix_web::test]
    async fn test_prices_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/prices").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body.is_object());
        assert!(body["BTC"].is_object());
        assert_eq!(body["BTC"]["asset"], "BTC");
    }
}
