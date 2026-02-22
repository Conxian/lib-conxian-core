#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use crate::api::config;
    use crate::engine::Engine;
    use serde_json::Value;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(config)
        ).await;
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
    }

    #[actix_web::test]
    async fn test_status_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(config)
        ).await;
        let req = test::TestRequest::get().uri("/api/v1/status").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "operational");
    }

    #[actix_web::test]
    async fn test_stacks_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(config)
        ).await;
        let req = test::TestRequest::get().uri("/api/v1/stacks").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "stacks");
        assert!(body["metadata"]["block_height"].is_string());
    }

    #[actix_web::test]
    async fn test_lightning_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(config)
        ).await;
        let req = test::TestRequest::get().uri("/api/v1/lightning").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "lightning");
        assert!(body["metadata"]["channel_count"].is_string());
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(config)
        ).await;
        let req = test::TestRequest::get().uri("/api/v1/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("gateway_uptime_seconds"));
    }
}
