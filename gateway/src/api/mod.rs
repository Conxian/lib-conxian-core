// pub mod mcp_handler;
#[cfg(test)]
mod tests;
use crate::engine::{
    Engine, PartnerLeadCreateInput, PartnerLeadStatus, PartnerLeadStatusUpdateInput,
    PartnerLeadTransitionError, ProposalExecutionError,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(bisq_handler)
            .service(rgb_handler)
            .service(bitvm_handler)
            .service(bitvm2_handler)
            .service(bitvm2_info_handler)
            .service(bitvm2_segments_handler)
            .service(bitvm2_verify_state_root_handler)
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
            .service(compliance_zkml_handler)
            .service(metrics_handler)
            .service(reserves_handler)
            .service(babylon_handler)
            .service(babylon_staking_handler)
            .service(bob_handler)
            .service(bob_info_handler)
            .service(merlin_handler)
            .service(merlin_stats_handler)
            .service(botanix_handler)
            .service(botanix_stats_handler)
            .service(b2network_handler)
            .service(citrea_handler)
            .service(bitlayer_handler)
            .service(bitlayer_info_handler)
            .service(alpen_handler)
            .service(alpen_stats_handler)
            .service(mezo_handler)
            .service(mezo_yield_handler)
            .service(zulu_handler)
            .service(zulu_info_handler)
            .service(bison_handler)
            .service(bison_stats_handler)
            .service(settlement_proposals_handler)
            .service(settlement_proposal_approve_handler)
            .service(settlement_proposal_execute_handler)
            .service(iso20022_handler)
            .service(papss_handler)
            .service(brics_handler)
            .service(hemi_handler)
            .service(hemi_status_handler)
            .service(taproot_assets_handler)
            .service(taproot_assets_stats_handler)
            .service(nubit_handler)
            .service(nubit_da_handler)
            .service(lorenzo_handler)
            .service(lorenzo_staking_handler)
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
            .service(core_dao_stats_handler)
            .service(risk_assessment_handler)
            // New aligned endpoints
            .service(financials_handler)
            .service(identity_handler)
            .service(erp_sync_handler)
            .service(cjcs_spec_handler)
            .service(dlc_bond_handler)
            .service(state_commit_handler)
            .service(sab_wallets_handler)
            .service(partner_intake_create_handler)
            .service(partner_intake_get_handler)
            .service(partner_intake_list_handler)
            .service(partner_intake_status_update_handler)
            // .service(mcp_handler::mcp_handler),
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

#[get("/bitvm2/info")]
async fn bitvm2_info_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_bitvm2_info();
    HttpResponse::Ok().json(res)
}
#[get("/bitvm2/segments/{state_root}")]
async fn bitvm2_segments_handler(
    engine: web::Data<Engine>,
    path: web::Path<String>,
) -> impl Responder {
    let state_root = path.into_inner();
    let res = engine.get_bitvm2_segments(&state_root);
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
pub struct Bitvm2VerifyStateRootRequest {
    pub state_root: String,
    pub proof: String,
    pub public_inputs: Option<Vec<String>>,
}

#[post("/bitvm2/verify-state-root")]
async fn bitvm2_verify_state_root_handler(
    engine: web::Data<Engine>,
    req: web::Json<Bitvm2VerifyStateRootRequest>,
) -> impl Responder {
    engine.increment_requests();

    let vk_b64 = match std::env::var(lib_conxian_core::bitvm2::ENV_BITVM2_GROTH16_VK_B64) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "state_root": req.state_root,
                "verified": false,
                "error": format!(
                    "{} is not configured",
                    lib_conxian_core::bitvm2::ENV_BITVM2_GROTH16_VK_B64
                ),
            }))
        }
    };

    match lib_conxian_core::bitvm2::verify_state_root_bn254_groth16(
        &vk_b64,
        &req.state_root,
        &req.proof,
        req.public_inputs.as_deref(),
    ) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "state_root": req.state_root,
            "verified": true,
            "proof_system": "groth16",
            "curve": "bn254"
        })),
        Ok(false) => HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "state_root": req.state_root,
            "verified": false,
            "error": "verification failed",
            "proof_system": "groth16",
            "curve": "bn254"
        })),
        Err(lib_conxian_core::bitvm2::Bitvm2VerifyError::InvalidVerifyingKey) => {
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "state_root": req.state_root,
                "verified": false,
                "error": "verifying key is not valid/configured",
            }))
        }
        Err(lib_conxian_core::bitvm2::Bitvm2VerifyError::Internal) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "state_root": req.state_root,
                "verified": false,
                "error": "internal verification error",
            }))
        }
        Err(err) => HttpResponse::BadRequest().json(serde_json::json!({
            "state_root": req.state_root,
            "verified": false,
            "error": err.to_string(),
        })),
    }
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
    let status = engine.get_status();
    HttpResponse::Ok().json(status)
}

#[get("/health")]
async fn health_handler(engine: web::Data<Engine>) -> impl Responder {
    if engine.is_healthy() {
        HttpResponse::Ok().json(serde_json::json!({ "status": "healthy", "engine": "active" }))
    } else {
        HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "status": "unhealthy", "engine": "starting" }))
    }
}

#[get("/compliance")]
async fn compliance_handler(engine: web::Data<Engine>) -> impl Responder {
    let compliance = engine.get_compliance_status();
    HttpResponse::Ok().json(compliance)
}

#[derive(Deserialize)]
pub struct ComplianceCheckRequest {
    pub address: String,
}

#[post("/compliance/check")]
async fn compliance_check_handler(
    engine: web::Data<Engine>,
    req: web::Json<ComplianceCheckRequest>,
) -> impl Responder {
    let res = engine.check_compliance(&req.address);
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
pub struct ZKMLVerifyRequest {
    pub proof: String,
}

#[post("/compliance/zkml-verify")]
async fn compliance_zkml_handler(
    engine: web::Data<Engine>,
    req: web::Json<ZKMLVerifyRequest>,
) -> impl Responder {
    let res = engine.verify_zkml_proof(&req.proof);
    HttpResponse::Ok().json(res)
}

#[get("/metrics")]
async fn metrics_handler(engine: web::Data<Engine>) -> impl Responder {
    let requests = engine
        .request_count
        .load(std::sync::atomic::Ordering::SeqCst);
    let tvl = *engine.total_tvl_usd.read().unwrap();
    let uptime = (chrono::Utc::now() - engine.start_time).num_seconds();

    let mut metrics = format!(
        "# HELP gateway_requests_total Total number of requests processed\n# TYPE gateway_requests_total counter\ngateway_requests_total {}\n",
        requests
    );
    metrics.push_str(&format!(
        "# HELP gateway_tvl_usd Total Value Locked in USD\n# TYPE gateway_tvl_usd gauge\ngateway_tvl_usd {:.2}\n",
        tvl
    ));
    metrics.push_str(&format!(
        "# HELP gateway_uptime_seconds System uptime in seconds\n# TYPE gateway_uptime_seconds gauge\ngateway_uptime_seconds {}\n",
        uptime
    ));

    HttpResponse::Ok().content_type("text/plain").body(metrics)
}

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

#[get("/bob/info")]
async fn bob_info_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_bob_info();
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
struct InvoiceRequest {
    amount_msat: u64,
    description: String,
}

#[post("/lightning/invoice")]
async fn lightning_invoice_handler(
    engine: web::Data<Engine>,
    req: web::Json<InvoiceRequest>,
) -> impl Responder {
    let res = engine.create_lightning_invoice(req.amount_msat, &req.description);
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
struct PayRequest {
    invoice: String,
    testnet: Option<bool>,
}

#[post("/lightning/pay")]
async fn lightning_pay_handler(
    engine: web::Data<Engine>,
    req: web::Json<PayRequest>,
) -> impl Responder {
    if !Engine::is_mainnet_only() && req.testnet.is_none() {
        return HttpResponse::Forbidden()
            .body("Mainnet-only endpoint. Use testnet flag for non-production validation.");
    }
    let res = engine.pay_lightning_invoice(&req.invoice);
    HttpResponse::Ok().json(res)
}

#[get("/stacks/contract/{id}")]
async fn stacks_contract_handler(
    engine: web::Data<Engine>,
    path: web::Path<String>,
) -> impl Responder {
    let res = engine.get_stacks_contract(&path.into_inner());
    HttpResponse::Ok().json(res)
}
#[get("/merlin")]
async fn merlin_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("merlin");
    HttpResponse::Ok().json(status)
}

#[get("/merlin/stats")]
async fn merlin_stats_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_merlin_stats();
    HttpResponse::Ok().json(res)
}

#[get("/botanix")]
async fn botanix_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("botanix");
    HttpResponse::Ok().json(status)
}

#[get("/botanix/stats")]
async fn botanix_stats_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_botanix_stats();
    HttpResponse::Ok().json(res)
}

#[get("/b2network")]
async fn b2network_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("b2network");
    HttpResponse::Ok().json(status)
}

#[get("/rgb/contract/{id}")]
async fn rgb_contract_handler(
    engine: web::Data<Engine>,
    path: web::Path<String>,
) -> impl Responder {
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

#[get("/bitlayer/info")]
async fn bitlayer_info_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_bitlayer_info();
    HttpResponse::Ok().json(res)
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
async fn changelly_rate_handler(
    engine: web::Data<Engine>,
    query: web::Query<RateRequest>,
) -> impl Responder {
    let res = engine.get_exchange_rate(&query.from, &query.to);
    HttpResponse::Ok().json(res)
}

#[get("/alpen")]
async fn alpen_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("alpen");
    HttpResponse::Ok().json(status)
}

#[get("/alpen/stats")]
async fn alpen_stats_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_alpen_stats();
    HttpResponse::Ok().json(res)
}

#[get("/mezo")]
async fn mezo_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("mezo");
    HttpResponse::Ok().json(status)
}

#[get("/mezo/yield")]
async fn mezo_yield_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_mezo_yield();
    HttpResponse::Ok().json(res)
}

#[get("/zulu")]
async fn zulu_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("zulu");
    HttpResponse::Ok().json(status)
}

#[get("/zulu/info")]
async fn zulu_info_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_zulu_info();
    HttpResponse::Ok().json(res)
}

#[get("/bison")]
async fn bison_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bison");
    HttpResponse::Ok().json(status)
}

#[get("/bison/stats")]
async fn bison_stats_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_bison_stats();
    HttpResponse::Ok().json(res)
}

#[get("/hemi")]
async fn hemi_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("hemi");
    HttpResponse::Ok().json(status)
}

#[get("/hemi/status")]
async fn hemi_status_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_hemi_status();
    HttpResponse::Ok().json(res)
}

#[get("/taproot-assets")]
async fn taproot_assets_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("taproot-assets");
    HttpResponse::Ok().json(status)
}

#[get("/taproot-assets/stats")]
async fn taproot_assets_stats_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_taproot_assets_stats();
    HttpResponse::Ok().json(res)
}

#[get("/nubit")]
async fn nubit_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("nubit");
    HttpResponse::Ok().json(status)
}

#[get("/nubit/da")]
async fn nubit_da_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_nubit_da_info();
    HttpResponse::Ok().json(res)
}

#[get("/lorenzo")]
async fn lorenzo_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("lorenzo");
    HttpResponse::Ok().json(status)
}

#[get("/lorenzo/stats")]
async fn lorenzo_staking_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_lorenzo_staking();
    HttpResponse::Ok().json(res)
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
async fn citrea_proof_handler(
    engine: web::Data<Engine>,
    path: web::Path<String>,
) -> impl Responder {
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

#[get("/core-dao/stats")]
async fn core_dao_stats_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_core_dao_stats();
    HttpResponse::Ok().json(res)
}

#[get("/risk-assessment")]
async fn risk_assessment_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let assessments = engine.get_risk_assessments();
    HttpResponse::Ok().json(assessments)
}

#[get("/financials")]
async fn financials_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_financial_metrics();
    HttpResponse::Ok().json(res)
}

#[get("/identity/{query}")]
async fn identity_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let res = engine.resolve_identity(&path.into_inner());
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
struct ErpSyncRequest {
    system: String,
    testnet: Option<bool>,
}

#[post("/erp/sync")]
async fn erp_sync_handler(
    engine: web::Data<Engine>,
    req: web::Json<ErpSyncRequest>,
) -> impl Responder {
    if false {
        return HttpResponse::Forbidden().finish();
    }
    let res = engine.sync_erp_data(&req.system);
    HttpResponse::Ok().json(res)
}

#[get("/spec/cjcs")]
async fn cjcs_spec_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_cjcs_v2_spec();
    HttpResponse::Ok().json(res)
}

#[get("/finance/bond/{id}")]
async fn dlc_bond_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let res = engine.get_dlc_bond_info(&path.into_inner());
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
struct StateCommitRequest {
    state_root: String,
    testnet: Option<bool>,
}

#[post("/state/commit")]
async fn state_commit_handler(
    engine: web::Data<Engine>,
    req: web::Json<StateCommitRequest>,
) -> impl Responder {
    if false {
        return HttpResponse::Forbidden().finish();
    }
    let res = engine.commit_state_to_tableland(&req.state_root);
    HttpResponse::Ok().json(res)
}

/// Partner intake endpoints require this header and env var pairing:
/// - Header: `X-Partner-Intake-Key`
/// - Env var: `PARTNER_INTAKE_API_KEY`
///
/// If the env var is absent/empty, handlers return `503 Service Unavailable`.
/// If the header is missing or mismatched, handlers return `401 Unauthorized`.
const PARTNER_INTAKE_API_KEY_ENV: &str = "PARTNER_INTAKE_API_KEY";
const PARTNER_INTAKE_API_KEY_HEADER: &str = "X-Partner-Intake-Key";
const PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

pub(crate) const GATEWAY_ADMIN_API_KEY_ENV: &str = "GATEWAY_ADMIN_API_KEY";
pub(crate) const GATEWAY_ADMIN_API_KEY_HEADER: &str = "X-Gateway-Admin-Key";

pub(crate) fn require_admin_auth(req: &HttpRequest) -> Result<(), HttpResponse> {
    let expected = match std::env::var(GATEWAY_ADMIN_API_KEY_ENV) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            return Err(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "gateway_admin_not_configured",
                "message": format!(
                    "{} must be set to enable administrative/governance APIs",
                    GATEWAY_ADMIN_API_KEY_ENV
                ),
            })))
        }
    };

    let provided = req
        .headers()
        .get(GATEWAY_ADMIN_API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match provided {
        Some(value) if value == expected => Ok(()),
        _ => Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": format!(
                "Provide {} header matching configured admin key",
                GATEWAY_ADMIN_API_KEY_HEADER
            ),
        }))),
    }
}

fn require_partner_intake_auth(req: &HttpRequest) -> Result<(), HttpResponse> {
    let expected = match std::env::var(PARTNER_INTAKE_API_KEY_ENV) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            return Err(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "partner_intake_not_configured",
                "message": format!(
                    "{} must be set to enable partner intake APIs",
                    PARTNER_INTAKE_API_KEY_ENV
                ),
            })))
        }
    };

    let provided = req
        .headers()
        .get(PARTNER_INTAKE_API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match provided {
        Some(value) if value == expected => Ok(()),
        _ => Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "unauthorized",
            "message": format!(
                "Provide {} header matching configured intake key",
                PARTNER_INTAKE_API_KEY_HEADER
            ),
        }))),
    }
}

fn require_idempotency_key(req: &HttpRequest) -> Result<String, HttpResponse> {
    req.headers()
        .get(PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "missing_idempotency_key",
                "message": format!(
                    "{} header is required for partner intake create requests",
                    PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER
                ),
            }))
        })
}

fn trim_optional(input: Option<String>) -> Option<String> {
    input
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[derive(Deserialize)]
struct PartnerLeadCreateRequest {
    partner_name: Option<String>,
    contact_name: Option<String>,
    contact_email: Option<String>,
    company_name: Option<String>,
    notes: Option<String>,
}

impl PartnerLeadCreateRequest {
    fn validate(self) -> Result<PartnerLeadCreateInput, Vec<String>> {
        let mut errors = Vec::new();

        let partner_name = trim_optional(self.partner_name).unwrap_or_else(|| {
            errors.push("partner_name is required".to_string());
            String::new()
        });

        let contact_name = trim_optional(self.contact_name).unwrap_or_else(|| {
            errors.push("contact_name is required".to_string());
            String::new()
        });

        let contact_email = trim_optional(self.contact_email).unwrap_or_else(|| {
            errors.push("contact_email is required".to_string());
            String::new()
        });

        if !contact_email.is_empty() && !contact_email.contains('@') {
            errors.push("contact_email must be a valid email address".to_string());
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(PartnerLeadCreateInput {
            partner_name,
            contact_name,
            contact_email,
            company_name: trim_optional(self.company_name),
            notes: trim_optional(self.notes),
        })
    }
}

#[derive(Deserialize)]
struct PartnerLeadListQuery {
    status: Option<PartnerLeadStatus>,
    owner: Option<String>,
}

#[derive(Deserialize)]
struct PartnerLeadStatusUpdateRequest {
    status: PartnerLeadStatus,
    owner: Option<String>,
    escalated_to: Option<String>,
    escalation_reason: Option<String>,
    event_note: Option<String>,
}

impl PartnerLeadStatusUpdateRequest {
    fn into_engine_input(self) -> PartnerLeadStatusUpdateInput {
        PartnerLeadStatusUpdateInput {
            status: self.status,
            owner: trim_optional(self.owner),
            escalated_to: trim_optional(self.escalated_to),
            escalation_reason: trim_optional(self.escalation_reason),
            event_note: trim_optional(self.event_note),
        }
    }
}

fn map_partner_transition_error(err: PartnerLeadTransitionError) -> HttpResponse {
    match err {
        PartnerLeadTransitionError::NotFound => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "lead_not_found"}))
        }
        PartnerLeadTransitionError::InvalidTransition { from, to } => HttpResponse::Conflict()
            .json(serde_json::json!({
                "error": "invalid_transition",
                "message": format!(
                    "Transition {} -> {} is not allowed",
                    from.as_str(),
                    to.as_str()
                ),
            })),
        PartnerLeadTransitionError::OwnerRequired => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_failed",
                "message": "owner is required for assigned/in_progress states",
            }))
        }
        PartnerLeadTransitionError::EscalationReasonRequired => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_failed",
                "message": "escalation_reason is required when moving to escalated",
            }))
        }
    }
}

#[post("/intake/partner")]
async fn partner_intake_create_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    payload: web::Json<PartnerLeadCreateRequest>,
) -> impl Responder {
    if let Err(response) = require_partner_intake_auth(&req) {
        return response;
    }

    let idempotency_key = match require_idempotency_key(&req) {
        Ok(key) => key,
        Err(response) => return response,
    };

    let input = match payload.into_inner().validate() {
        Ok(input) => input,
        Err(errors) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_failed",
                "details": errors,
            }))
        }
    };

    let outcome = engine.create_partner_lead(input, &idempotency_key);
    if outcome.idempotent_replay {
        HttpResponse::Ok().json(outcome)
    } else {
        HttpResponse::Created().json(outcome)
    }
}

#[get("/intake/partner/{id}")]
async fn partner_intake_get_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    if let Err(response) = require_partner_intake_auth(&req) {
        return response;
    }

    let lead_id = path.into_inner();
    match engine.get_partner_lead(&lead_id) {
        Some(lead) => HttpResponse::Ok().json(lead),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "lead_not_found",
            "lead_id": lead_id,
        })),
    }
}

#[get("/intake/partner")]
async fn partner_intake_list_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    query: web::Query<PartnerLeadListQuery>,
) -> impl Responder {
    if let Err(response) = require_partner_intake_auth(&req) {
        return response;
    }

    let leads = engine.list_partner_leads(query.status.clone(), query.owner.as_deref());
    HttpResponse::Ok().json(leads)
}

#[post("/intake/partner/{id}/status")]
async fn partner_intake_status_update_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    path: web::Path<String>,
    payload: web::Json<PartnerLeadStatusUpdateRequest>,
) -> impl Responder {
    if let Err(response) = require_partner_intake_auth(&req) {
        return response;
    }

    let lead_id = path.into_inner();
    match engine.transition_partner_lead(&lead_id, payload.into_inner().into_engine_input()) {
        Ok(lead) => HttpResponse::Ok().json(lead),
        Err(err) => map_partner_transition_error(err),
    }
}

#[get("/settlement/proposals")]
async fn settlement_proposals_handler(engine: web::Data<Engine>) -> impl Responder {
    let res = engine.get_proposals();
    HttpResponse::Ok().json(res)
}

#[derive(Deserialize)]
struct SettlementMutationRequest {
    testnet: Option<bool>,
}

#[post("/settlement/proposals/{id}/approve")]
async fn settlement_proposal_approve_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    path: web::Path<String>,
    _query: web::Query<SettlementMutationRequest>,
) -> impl Responder {
    if let Err(response) = require_admin_auth(&req) {
        return response;
    }

    if false {
        return HttpResponse::Forbidden().finish();
    }

    let proposal_id = path.into_inner();
    if engine.approve_proposal(&proposal_id) {
        HttpResponse::Ok().json(serde_json::json!({"status": "Approved"}))
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Proposal not found or not Pending"}))
    }
}

#[post("/settlement/proposals/{id}/execute")]
async fn settlement_proposal_execute_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    path: web::Path<String>,
    _query: web::Query<SettlementMutationRequest>,
) -> impl Responder {
    if let Err(response) = require_admin_auth(&req) {
        return response;
    }

    if false {
        return HttpResponse::Forbidden().finish();
    }

    let proposal_id = path.into_inner();
    match engine.execute_proposal(&proposal_id) {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({"status": "Executed"})),
        Err(ProposalExecutionError::NotFound | ProposalExecutionError::NotApproved) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Proposal not found or not Approved",
            }))
        }
        Err(ProposalExecutionError::TimelockNotExpired {
            current_block,
            timelock_end_block,
        }) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Timelock not expired",
            "message": format!(
                "Proposal {} cannot be executed before block {} (current block {}).",
                proposal_id, timelock_end_block, current_block
            ),
            "current_block": current_block,
            "timelock_end_block": timelock_end_block,
        })),
    }
}

#[post("/settlement/iso20022")]
async fn iso20022_handler(engine: web::Data<Engine>, payload: web::Json<Value>) -> impl Responder {
    if false {
        return HttpResponse::Forbidden().finish();
    }
    let res = engine.process_external_settlement("ISO20022", payload.into_inner());
    HttpResponse::Ok().json(res)
}

#[post("/settlement/papss")]
async fn papss_handler(engine: web::Data<Engine>, payload: web::Json<Value>) -> impl Responder {
    if false {
        return HttpResponse::Forbidden().finish();
    }
    let res = engine.process_external_settlement("PAPSS", payload.into_inner());
    HttpResponse::Ok().json(res)
}

#[post("/settlement/brics")]
async fn brics_handler(engine: web::Data<Engine>, payload: web::Json<Value>) -> impl Responder {
    if false {
        return HttpResponse::Forbidden().finish();
    }
    let res = engine.process_external_settlement("BRICS", payload.into_inner());
    HttpResponse::Ok().json(res)
}

#[get("/sab/wallets")]
async fn sab_wallets_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let wallets = engine.get_sab_wallets();
    HttpResponse::Ok().json(wallets)
}
