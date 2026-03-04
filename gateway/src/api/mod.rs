use actix_web::{get, post, web, HttpResponse, Responder};
use crate::engine::Engine;
use serde::Deserialize;
use std::sync::atomic::Ordering;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(bisq_handler)
            .service(rgb_handler)
            .service(bitvm_handler)
            .service(bitvm2_handler)
            .service(changelly_handler)
            .service(changelly_rate_handler)
            .service(stacks_handler)
            .service(lightning_handler)
            .service(liquid_handler)
            .service(liquid_peg_handler)
            .service(rootstock_handler)
            .service(rootstock_powpeg_handler)
            .service(layers_handler)
            .service(status_handler)
            .service(health_handler)
            .service(compliance_handler)
            .service(compliance_check_handler)
            .service(metrics_handler)
            .service(reserves_handler)
            .service(babylon_handler)
            .service(babylon_staking_handler)
            .service(bob_handler)
            .service(merlin_handler)
            .service(botanix_handler)
            .service(b2network_handler)
            .service(citrea_handler)
            .service(bitlayer_handler)
            .service(alpen_handler)
            .service(mezo_handler)
            .service(zulu_handler)
            .service(bison_handler)
            .service(hemi_handler)
            .service(taproot_assets_handler)
            .service(nubit_handler)
            .service(lorenzo_handler)
            .service(core_dao_handler)
            .service(prices_handler)
            .service(lightning_invoice_handler)
            .service(lightning_pay_handler)
            .service(stacks_contract_handler)
            .service(rgb_contract_handler)
            .service(bitvm_proof_handler)
            .service(b2network_status_handler)
            .service(citrea_proof_handler)
            .service(affiliates_handler)
            .service(marketing_handler)
            .service(risk_assessment_handler)
    );
}

#[get("/reserves")]
async fn reserves_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let reserves = engine.get_reserves();
    HttpResponse::Ok().json(reserves)
}

#[get("/bisq")]
async fn bisq_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bisq");
    HttpResponse::Ok().json(status)
}

#[get("/rgb")]
async fn rgb_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("rgb");
    HttpResponse::Ok().json(status)
}

#[get("/bitvm")]
async fn bitvm_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bitvm");
    HttpResponse::Ok().json(status)
}

#[get("/bitvm2")]
async fn bitvm2_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bitvm2");
    HttpResponse::Ok().json(status)
}

#[get("/changelly")]
async fn changelly_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("changelly");
    HttpResponse::Ok().json(status)
}

#[get("/status")]
async fn status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(engine.get_system_info())
}

#[get("/health")]
async fn health_handler(engine: web::Data<Engine>) -> impl Responder {
    if engine.is_healthy() {
        HttpResponse::Ok().json(serde_json::json!({ "status": "healthy", "engine": "active" }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({ "status": "degraded", "engine": "stale" }))
    }
}

#[get("/compliance")]
async fn compliance_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let compliance = engine.get_compliance_status();
    HttpResponse::Ok().json(compliance)
}

#[derive(Deserialize)]
pub struct ComplianceCheckRequest {
    pub address: String,
}

#[post("/compliance/check")]
async fn compliance_check_handler(engine: web::Data<Engine>, req: web::Json<ComplianceCheckRequest>) -> impl Responder {
    let res = engine.check_compliance(&req.address);
    HttpResponse::Ok().json(res)
}

#[get("/metrics")]
async fn metrics_handler(engine: web::Data<Engine>) -> impl Responder {
    let uptime = chrono::Utc::now().signed_duration_since(engine.start_time).num_seconds();
    let requests = engine.request_count.load(Ordering::SeqCst);
    let tvl = engine.total_tvl_usd.load(Ordering::SeqCst);
    let nodes = engine.active_sovereign_nodes.load(Ordering::SeqCst);

    let mut metrics = format!(
        "# HELP gateway_uptime_seconds Uptime in seconds\n# TYPE gateway_uptime_seconds counter\ngateway_uptime_seconds {}\n# HELP gateway_requests_total Total number of requests processed\n# TYPE gateway_requests_total counter\ngateway_requests_total {}\n# HELP gateway_tvl_usd Total Value Locked in USD\n# TYPE gateway_tvl_usd gauge\ngateway_tvl_usd {}\n# HELP gateway_active_nodes Number of active sovereign nodes\n# TYPE gateway_active_nodes gauge\ngateway_active_nodes {}\n",
        uptime, requests, tvl, nodes
    );

    let statuses = engine.get_all_service_statuses();
    for status in statuses.values() {
        metrics.push_str(&format!(
            "# HELP gateway_service_latency_ms Latency of {} in ms\ngateway_service_latency_ms{{service=\"{}\"}} {}\n",
            status.name, status.name, status.latency_ms
        ));
        let risk_score = match status.risk_level.as_str() {
            "Low" => 10,
            "Medium" => 50,
            "High" => 90,
            _ => 0,
        };
        metrics.push_str(&format!(
            "# HELP gateway_service_risk_score Risk score of {}\ngateway_service_risk_score{{service=\"{}\"}} {}\n",
            status.name, status.name, risk_score
        ));
    }

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics)
}

#[cfg(test)]
mod tests;

#[get("/stacks")]
async fn stacks_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("stacks");
    HttpResponse::Ok().json(status)
}

#[get("/lightning")]
async fn lightning_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("lightning");
    HttpResponse::Ok().json(status)
}

#[get("/liquid")]
async fn liquid_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("liquid");
    HttpResponse::Ok().json(status)
}

#[get("/liquid/peg")]
async fn liquid_peg_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_liquid_peg();
    HttpResponse::Ok().json(res)
}

#[get("/rootstock")]
async fn rootstock_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("rootstock");
    HttpResponse::Ok().json(status)
}

#[get("/rootstock/powpeg")]
async fn rootstock_powpeg_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_rootstock_powpeg();
    HttpResponse::Ok().json(res)
}

#[get("/layers")]
async fn layers_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let statuses = engine.get_all_service_statuses();
    HttpResponse::Ok().json(statuses)
}

#[get("/babylon")]
async fn babylon_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("babylon");
    HttpResponse::Ok().json(status)
}

#[get("/babylon/staking")]
async fn babylon_staking_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_babylon_staking();
    HttpResponse::Ok().json(res)
}

#[get("/bob")]
async fn bob_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bob");
    HttpResponse::Ok().json(status)
}

#[derive(Deserialize)]
struct InvoiceRequest {
    amount_msat: u64,
    description: String,
}

#[post("/lightning/invoice")]
async fn lightning_invoice_handler(engine: web::Data<Engine>, req: web::Json<InvoiceRequest>) -> impl Responder {
    let res = engine.create_lightning_invoice(req.amount_msat, &req.description);
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
struct PayRequest {
    invoice: String,
}

#[post("/lightning/pay")]
async fn lightning_pay_handler(engine: web::Data<Engine>, req: web::Json<PayRequest>) -> impl Responder {
    let res = engine.pay_lightning_invoice(&req.invoice);
    HttpResponse::Ok().json(res)
}

#[get("/stacks/contract/{id}")]
async fn stacks_contract_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let res = engine.get_stacks_contract(&path.into_inner());
    HttpResponse::Ok().json(res)
}
#[get("/merlin")]
async fn merlin_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("merlin");
    HttpResponse::Ok().json(status)
}

#[get("/botanix")]
async fn botanix_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("botanix");
    HttpResponse::Ok().json(status)
}

#[get("/b2network")]
async fn b2network_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("b2network");
    HttpResponse::Ok().json(status)
}

#[get("/rgb/contract/{id}")]
async fn rgb_contract_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let res = engine.get_rgb_contract(&path.into_inner());
    HttpResponse::Ok().json(res)
}

#[get("/bitvm/proof/{id}")]
async fn bitvm_proof_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let res = engine.get_bitvm_proof(&path.into_inner());
    HttpResponse::Ok().json(res)
}

#[get("/citrea")]
async fn citrea_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("citrea");
    HttpResponse::Ok().json(status)
}

#[get("/bitlayer")]
async fn bitlayer_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bitlayer");
    HttpResponse::Ok().json(status)
}

#[get("/prices")]
async fn prices_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let prices = engine.get_prices();
    HttpResponse::Ok().json(prices)
}

#[derive(Deserialize)]
struct RateRequest {
    from: String,
    to: String,
}

#[get("/changelly/rate")]
async fn changelly_rate_handler(engine: web::Data<Engine>, query: web::Query<RateRequest>) -> impl Responder {
    let res = engine.get_exchange_rate(&query.from, &query.to);
    HttpResponse::Ok().json(res)
}

#[get("/alpen")]
async fn alpen_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("alpen");
    HttpResponse::Ok().json(status)
}

#[get("/mezo")]
async fn mezo_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("mezo");
    HttpResponse::Ok().json(status)
}

#[get("/zulu")]
async fn zulu_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("zulu");
    HttpResponse::Ok().json(status)
}

#[get("/bison")]
async fn bison_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bison");
    HttpResponse::Ok().json(status)
}

#[get("/hemi")]
async fn hemi_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("hemi");
    HttpResponse::Ok().json(status)
}

#[get("/taproot-assets")]
async fn taproot_assets_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("taproot-assets");
    HttpResponse::Ok().json(status)
}

#[get("/nubit")]
async fn nubit_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("nubit");
    HttpResponse::Ok().json(status)
}

#[get("/lorenzo")]
async fn lorenzo_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("lorenzo");
    HttpResponse::Ok().json(status)
}

#[get("/core-dao")]
async fn core_dao_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("core-dao");
    HttpResponse::Ok().json(status)
}

#[get("/b2network/status")]
async fn b2network_status_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_b2_status();
    HttpResponse::Ok().json(res)
}

#[get("/citrea/proof/{id}")]
async fn citrea_proof_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let res = engine.get_citrea_proof(&path.into_inner());
    HttpResponse::Ok().json(res)
}

#[get("/affiliates")]
async fn affiliates_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let affiliates = engine.get_affiliates();
    HttpResponse::Ok().json(affiliates)
}

#[get("/marketing")]
async fn marketing_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let marketing = engine.get_marketing();
    HttpResponse::Ok().json(marketing)
}

#[get("/risk-assessment")]
async fn risk_assessment_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let assessments = engine.get_risk_assessments();
    HttpResponse::Ok().json(assessments)
}
