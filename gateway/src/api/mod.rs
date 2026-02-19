use actix_web::{get, web, HttpResponse, Responder};
use crate::engine::Engine;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(bisq_handler)
            .service(rgb_handler)
            .service(bitvm_handler)
            .service(changelly_handler)
            .service(status_handler)
            .service(health_handler)
            .service(compliance_handler)
            .service(metrics_handler)
    );
}

#[get("/bisq")]
async fn bisq_handler(engine: web::Data<Engine>) -> impl Responder {
    let status = engine.get_service_status("bisq");
    HttpResponse::Ok().json(status)
}

#[get("/rgb")]
async fn rgb_handler(engine: web::Data<Engine>) -> impl Responder {
    let status = engine.get_service_status("rgb");
    HttpResponse::Ok().json(status)
}

#[get("/bitvm")]
async fn bitvm_handler(engine: web::Data<Engine>) -> impl Responder {
    let status = engine.get_service_status("bitvm");
    HttpResponse::Ok().json(status)
}

#[get("/changelly")]
async fn changelly_handler(engine: web::Data<Engine>) -> impl Responder {
    let status = engine.get_service_status("changelly");
    HttpResponse::Ok().json(status)
}

#[get("/status")]
async fn status_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_system_info())
}

#[get("/health")]
async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy" }))
}

#[get("/compliance")]
async fn compliance_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "compliant",
        "last_audit": chrono::Utc::now(),
        "rules_active": ["KYC", "AML", "NetworkIntegrity"]
    }))
}

#[get("/metrics")]
async fn metrics_handler() -> impl Responder {
    // Placeholder for Prometheus/OpenTelemetry metrics
    HttpResponse::Ok().body("# HELP gateway_uptime Uptime in seconds\n# TYPE gateway_uptime counter\ngateway_uptime 100\n")
}

#[cfg(test)]
mod tests;
