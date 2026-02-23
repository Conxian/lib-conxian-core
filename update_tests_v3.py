with open('gateway/src/api/tests.rs', 'r') as f:
    tests = f.read()

new_test = """
    #[actix_web::test]
    async fn test_changelly_rate_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/changelly/rate?from=BTC&to=USD").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["from"], "BTC");
        assert_eq!(body["to"], "USD");
        assert!(body["rate"].is_number());
    }

    #[actix_web::test]
    async fn test_health_check_dynamic() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["engine"], "active");
    }
"""

if 'test_changelly_rate_endpoint' not in tests:
    last_brace_idx = tests.rfind('}')
    tests = tests[:last_brace_idx] + new_test + tests[last_brace_idx:]
    with open('gateway/src/api/tests.rs', 'w') as f:
        f.write(tests)
