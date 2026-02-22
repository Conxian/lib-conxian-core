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
}
