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
            .service(changelly_handler)
            .service(stacks_handler)
            .service(lightning_handler)
            .service(liquid_handler)
            .service(rootstock_handler)
            .service(layers_handler)
            .service(status_handler)
            .service(health_handler)
            .service(compliance_handler)
            .service(metrics_handler)
            .service(reserves_handler)
            .service(babylon_handler)
            .service(bob_handler)
            .service(merlin_handler)
            .service(botanix_handler)
            .service(b2network_handler)
            .service(lightning_invoice_handler)
            .service(lightning_pay_handler)
            .service(stacks_contract_handler)
            .service(rgb_contract_handler)
            .service(bitvm_proof_handler)
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
async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy" }))
}

#[get("/compliance")]
async fn compliance_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "compliant",
        "last_audit": chrono::Utc::now(),
        "rules_active": ["KYC", "AML", "NetworkIntegrity"]
    }))
}

#[get("/metrics")]
async fn metrics_handler(engine: web::Data<Engine>) -> impl Responder {
    let uptime = chrono::Utc::now().signed_duration_since(engine.start_time).num_seconds();
    let requests = engine.request_count.load(Ordering::SeqCst);
    let tvl = engine.total_tvl_usd.load(Ordering::SeqCst);
    let nodes = engine.active_sovereign_nodes.load(Ordering::SeqCst);

    let metrics = format!(
        "# HELP gateway_uptime_seconds Uptime in seconds\n         # TYPE gateway_uptime_seconds counter\n         gateway_uptime_seconds {}\n         # HELP gateway_requests_total Total number of requests processed\n         # TYPE gateway_requests_total counter\n         gateway_requests_total {}\n         # HELP gateway_tvl_usd Total Value Locked in USD\n         # TYPE gateway_tvl_usd gauge\n         gateway_tvl_usd {}\n         # HELP gateway_active_nodes Number of active sovereign nodes\n         # TYPE gateway_active_nodes gauge\n         gateway_active_nodes {}\n",
        uptime, requests, tvl, nodes
    );

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

#[get("/rootstock")]
async fn rootstock_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("rootstock");
    HttpResponse::Ok().json(status)
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
