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
    }

    #[actix_web::test]
    async fn test_rgb_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/rgb").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "rgb");
    }

    #[actix_web::test]
    async fn test_bitvm_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bitvm").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "bitvm");
    }

    #[actix_web::test]
    async fn test_changelly_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/changelly").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "changelly");
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
    }

    #[actix_web::test]
    async fn test_lightning_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/lightning").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "lightning");
    }

    #[actix_web::test]
    async fn test_liquid_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/liquid").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "liquid");
    }

    #[actix_web::test]
    async fn test_rootstock_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/rootstock").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "rootstock");
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
    }

    #[actix_web::test]
    async fn test_babylon_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/babylon").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_bob_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bob").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_merlin_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/merlin").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_botanix_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/botanix").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_b2network_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/b2network").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_citrea_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/citrea").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_bitlayer_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bitlayer").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_alpen_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/alpen").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_mezo_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/mezo").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_zulu_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/zulu").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_bison_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bison").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_hemi_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/hemi").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_taproot_assets_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/taproot-assets").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_nubit_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/nubit").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_lorenzo_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/lorenzo").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_risk_assessment_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/risk-assessment").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body.is_object());
    }

    #[actix_web::test]
    async fn test_core_dao_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/core-dao/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["satoshi_plus_status"], "Active");
    }

    #[actix_web::test]
    async fn test_lorenzo_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/lorenzo/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["reward_token"], "stBTC");
    }

    #[actix_web::test]
    async fn test_hemi_status_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/hemi/status").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["sequencer_status"], "Active");
    }

    #[actix_web::test]
    async fn test_bob_info_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bob/info").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body["connected_chains"].is_array());
    }

    #[actix_web::test]
    async fn test_merlin_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/merlin/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["zk_proving_status"], "Active");
    }

    #[actix_web::test]
    async fn test_mezo_yield_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/mezo/yield").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body["current_yield_apy"].is_number());
    }

    #[actix_web::test]
    async fn test_nubit_da_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/nubit/da").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body["da_throughput_mbps"].is_number());
    }

    #[actix_web::test]
    async fn test_bison_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bison/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body["zk_roll_uptime_pct"].is_number());
    }

    #[actix_web::test]
    async fn test_zulu_info_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/zulu/info").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["layer_type"], "Multi-layer");
    }

    #[actix_web::test]
    async fn test_botanix_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/botanix/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["spiderchain_nodes"], 144);
    }

    #[actix_web::test]
    async fn test_bitlayer_info_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bitlayer/info").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["bitvm_challenge_status"], "Healthy");
    }

    #[actix_web::test]
    async fn test_alpen_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/alpen/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["zk_proof_type"], "SNARK");
    }

    #[actix_web::test]
    async fn test_taproot_assets_stats_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/taproot-assets/stats").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["lightning_integration"], "Enabled");
    }

    #[actix_web::test]
    async fn test_bitvm2_info_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bitvm2/info").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["paradigm"], "ZK-Fraud Proofs");
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
    }

    #[actix_web::test]
    async fn test_compliance_check_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/compliance/check")
            .set_json(serde_json::json!({"address": "bc1qsafe"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["compliant"], true);
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
    async fn test_lightning_pay_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/lightning/pay")
            .set_json(serde_json::json!({"invoice": "lnbc123"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "success");
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
    async fn test_bitvm_proof_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/bitvm/proof/PROOF123").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["proof_id"], "PROOF123");
        assert_eq!(body["status"], "Verified");
    }

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
    async fn test_b2network_status_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/b2network/status").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["proof_status"], "Verified");
    }

    #[actix_web::test]
    async fn test_citrea_proof_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/citrea/proof/BATCH123").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["batch_id"], "BATCH123");
        assert_eq!(body["status"], "Finalized");
    }

    #[actix_web::test]
    async fn test_liquid_peg_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/liquid/peg").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["asset"], "L-BTC");
    }

    #[actix_web::test]
    async fn test_rootstock_powpeg_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/rootstock/powpeg").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["asset"], "RBTC");
    }

    #[actix_web::test]
    async fn test_babylon_staking_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/babylon/staking").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body["staked_btc"].is_string());
    }

    #[actix_web::test]
    async fn test_affiliates_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/affiliates").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body.is_object());
    }

    #[actix_web::test]
    async fn test_marketing_endpoint() {
        let engine = web::Data::new(Engine::new());
        let app = test::init_service(App::new().app_data(engine).configure(config)).await;
        let req = test::TestRequest::get().uri("/api/v1/marketing").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = test::read_body_json(resp).await;
        assert!(body.is_array());
    }
}
