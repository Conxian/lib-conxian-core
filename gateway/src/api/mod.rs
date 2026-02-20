use actix_web::{get, web, HttpResponse, Responder};
use crate::engine::Engine;
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
            .service(status_handler)
            .service(health_handler)
            .service(compliance_handler)
            .service(metrics_handler)
    );
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
    // Health check doesn't necessarily need to increment request count for metrics
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

    let metrics = format!(
        "# HELP gateway_uptime_seconds Uptime in seconds\n         # TYPE gateway_uptime_seconds counter\n         gateway_uptime_seconds {}\n         # HELP gateway_requests_total Total number of requests processed\n         # TYPE gateway_requests_total counter\n         gateway_requests_total {}\n",
        uptime, requests
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
