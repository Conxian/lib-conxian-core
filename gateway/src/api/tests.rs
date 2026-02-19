#[cfg(test)]
mod tests {
    use actix_web::{test, App, web};
    use crate::api;
    use crate::engine::Engine;

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
    }
}
