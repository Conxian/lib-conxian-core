# Update API.md
with open('docs/API.md', 'r') as f:
    api = f.read()

if 'ComplianceStatus' not in api:
    api += """
### ComplianceStatus
```json
{
  "status": "compliant",
  "last_audit": "2024-05-20T10:00:00Z",
  "rules_active": ["KYC", "AML", "NetworkIntegrity"],
  "risk_score": 15
}
```
"""
    with open('docs/API.md', 'w') as f:
        f.write(api)

# Update tests.rs
with open('gateway/src/api/tests.rs', 'r') as f:
    tests = f.read()

new_test = """
    #[actix_web::test]
    async fn test_compliance_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/compliance").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "compliant");
        assert!(body["risk_score"].is_number());
    }
"""

if 'test_compliance_endpoint' not in tests:
    # Insert before the last closing brace
    last_brace_idx = tests.rfind('}')
    tests = tests[:last_brace_idx] + new_test + tests[last_brace_idx:]
    with open('gateway/src/api/tests.rs', 'w') as f:
        f.write(tests)
