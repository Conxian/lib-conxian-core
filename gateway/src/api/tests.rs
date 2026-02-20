#[cfg(test)]
mod tests {
    use actix_web::{test, App, web};
    use crate::api;
    use crate::engine::Engine;
    use serde_json::Value;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
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
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/status").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "operational");
    }

    #[actix_web::test]
    async fn test_bisq_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/bisq").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "bisq");
        assert_eq!(body["data_availability"], "On-chain");
    }

    #[actix_web::test]
    async fn test_stacks_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/stacks").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "stacks");
        assert_eq!(body["trust_model"], "PoX");
        assert_eq!(body["bridge_security"], "sBTC Bridge");
    }

    #[actix_web::test]
    async fn test_lightning_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/lightning").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "lightning");
        assert_eq!(body["trust_model"], "State Channels");
    }

    #[actix_web::test]
    async fn test_liquid_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/liquid").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "liquid");
        assert_eq!(body["bridge_security"], "Strong Federation");
    }

    #[actix_web::test]
    async fn test_rootstock_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/rootstock").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "rootstock");
        assert_eq!(body["trust_model"], "Powpeg");
    }

    #[actix_web::test]
    async fn test_compliance_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req = test::TestRequest::get().uri("/api/v1/compliance").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "compliant");
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(
            App::new()
                .app_data(engine.clone())
                .configure(api::config)
        ).await;

        let req1 = test::TestRequest::get().uri("/api/v1/status").to_request();
        test::call_service(&app, req1).await;

        let req = test::TestRequest::get().uri("/api/v1/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("gateway_requests_total 1"));
    }
}
