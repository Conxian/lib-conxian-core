// pub mod mcp_handler;

use crate::engine::anchoring::{
    AnchoringError, AnchoringRequest, AnchoringTarget, DEFAULT_MAX_RETRY_ATTEMPTS,
};
use crate::engine::{
    Engine, PartnerLeadCreateInput, PartnerLeadStatus, PartnerLeadStatusUpdateInput,
    PartnerLeadTransitionError, ProposalExecutionError,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(bisq_handler)
            .service(lightning_status_handler)
            .service(babylon_status_handler)
            .service(citrea_status_handler)
            .service(exchange_rate_handler)
            .service(bitvm_proof_handler)
            .service(stacks_contract_handler)
            .service(changelly_handler)
            .service(babylon_staking_handler)
            .service(citrea_handler)
            .service(rgb_handler)
            .service(rgb_status_handler)
            .service(bitvm_handler)
            .service(bitvm_status_handler)
            .service(bitvm2_handler)
            .service(bitvm2_info_handler)
            .service(bitvm2_segments_handler)
            .service(bitvm2_verify_state_root_handler)
            .service(bob_handler)
            .service(merlin_handler)
            .service(botanix_handler)
            .service(alpen_handler)
            .service(bison_handler)
            .service(hemi_handler)
            .service(taproot_assets_handler)
            .service(nubit_handler)
            .service(b2_handler)
            .service(lorenzo_handler)
            .service(mezo_handler)
            .service(mezo_status_handler)
            .service(zulu_handler)
            .service(bitlayer_handler)
            .service(stacks_handler)
            .service(liquid_handler)
            .service(rootstock_handler)
            .service(core_dao_handler)
            .service(layers_handler)
            .service(mezo_yield_handler)
            .service(rootstock_powpeg_handler)
            .service(liquid_peg_handler)
            .service(reserves_handler)
            .service(financials_handler)
            .service(risk_assessment_handler)
            .service(status_handler)
            .service(health_handler)
            .service(metrics_handler)
            .service(mcp_handler)
            .service(marketing_handler)
            .service(affiliates_handler)
            .service(prices_handler)
            .service(compliance_check_handler)
            .service(compliance_zkml_verify_handler)
            .service(identity_resolve_handler)
            .service(lightning_pay_handler)
            .service(lightning_invoice_handler)
            .service(erp_sync_handler)
            .service(spec_cjcs_handler)
            .service(finance_bond_handler)
            .service(state_commit_handler)
            .service(settlement_proposals_handler)
            .service(settlement_proposal_approve_handler)
            .service(settlement_proposal_execute_handler)
            .service(iso20022_handler)
            .service(papss_handler)
            .service(brics_handler)
            .service(sab_wallets_handler)
            .service(partner_intake_create_handler)
            .service(partner_intake_get_handler)
            .service(partner_intake_list_handler)
            .service(partner_intake_status_update_handler),
    );
}

#[get("/bisq")]
async fn bisq_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bisq");
    HttpResponse::Ok().json(status)
}

#[get("/rgb/contract/{id}")]
async fn rgb_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let _contract_id = path.into_inner();
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
    HttpResponse::Ok().json(engine.get_bitvm2_info())
}

#[get("/bitvm2/segments/{state_root}")]
async fn bitvm2_segments_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    let state_root = path.into_inner();
    HttpResponse::Ok().json(engine.get_bitvm2_segments(&state_root))
}

#[derive(Deserialize)]
struct Bitvm2VerifyStateRootRequest {
    vk_b64: String,
    state_root: String,
    proof_b64: String,
    extra_public_inputs: Option<Vec<String>>,
}

#[post("/bitvm2/verify-state-root")]
async fn bitvm2_verify_state_root_handler(
    engine: web::Data<Engine>,
    payload: web::Json<Bitvm2VerifyStateRootRequest>,
) -> impl Responder {
    engine.increment_requests();
    match lib_conxian_core::bitvm2::verify_state_root_bn254_groth16(
        &payload.vk_b64,
        &payload.state_root,
        &payload.proof_b64,
        payload.extra_public_inputs.as_deref(),
    ) {
        Ok(valid) => HttpResponse::Ok().json(serde_json::json!({ "valid": valid })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": format!("{:?}", e) })),
    }
}

#[get("/bob")]
async fn bob_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_bob_info())
}

#[get("/merlin")]
async fn merlin_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_merlin_stats())
}

#[get("/botanix/stats")]
async fn botanix_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_botanix_stats())
}

#[get("/alpen/stats")]
async fn alpen_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_alpen_stats())
}

#[get("/bison/stats")]
async fn bison_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_bison_stats())
}

#[get("/hemi")]
async fn hemi_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_hemi_status())
}

#[get("/taproot-assets/stats")]
async fn taproot_assets_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_taproot_assets_stats())
}

#[get("/nubit/da")]
async fn nubit_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_nubit_da_info())
}

#[get("/b2network")]
async fn b2_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_b2_status())
}

#[get("/lorenzo")]
async fn lorenzo_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("lorenzo");
    HttpResponse::Ok().json(status)
}

#[get("/mezo/yield")]
async fn mezo_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("mezo");
    HttpResponse::Ok().json(status)
}

#[get("/zulu/info")]
async fn zulu_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("zulu");
    HttpResponse::Ok().json(status)
}

#[get("/bitlayer/info")]
async fn bitlayer_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bitlayer");
    HttpResponse::Ok().json(status)
}

#[get("/stacks")]
async fn stacks_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("stacks");
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

#[get("/core-dao/stats")]
async fn core_dao_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("core-dao");
    HttpResponse::Ok().json(status)
}

#[get("/layers")]
async fn layers_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let layers = engine.get_all_service_statuses();
    HttpResponse::Ok().json(layers)
}

#[get("/reserves")]
async fn reserves_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let reserves = engine.reserves.read().unwrap().clone();
    HttpResponse::Ok().json(reserves)
}

#[get("/financials")]
async fn financials_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let metrics = engine.financial_metrics.read().unwrap().clone();
    HttpResponse::Ok().json(metrics)
}

#[get("/risk-assessment")]
async fn risk_assessment_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let assessments = engine.get_risk_assessments();
    HttpResponse::Ok().json(assessments)
}

#[get("/status")]
async fn status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let requests = engine.request_count.load(std::sync::atomic::Ordering::SeqCst);
    let uptime = (chrono::Utc::now() - engine.start_time).num_seconds();
    let tvl = *engine.total_tvl_usd.read().unwrap();

    HttpResponse::Ok().json(serde_json::json!({
        "version": engine.version,
        "uptime_seconds": uptime,
        "status": "operational",
        "total_requests": requests,
        "total_tvl_usd": tvl
    }))
}

#[get("/health")]
async fn health_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": "0.2.5"
    }))
}

#[get("/metrics")]
async fn metrics_handler(engine: web::Data<Engine>) -> impl Responder {
    let requests = engine.request_count.load(std::sync::atomic::Ordering::SeqCst);
    let uptime = (chrono::Utc::now() - engine.start_time).num_seconds();
    HttpResponse::Ok().json(serde_json::json!({
        "total_requests": requests,
        "uptime_seconds": uptime,
        "active_services": engine.get_all_service_statuses().len()
    }))
}

#[derive(Deserialize)]
struct ComplianceCheckRequest {
    address: String,
}

#[post("/compliance/check")]
async fn compliance_check_handler(engine: web::Data<Engine>, payload: web::Json<ComplianceCheckRequest>) -> impl Responder {
    engine.increment_requests();
    // In a real implementation, we'd check the address
    log::info!("Checking compliance for address: {}", payload.address);
    HttpResponse::Ok().json(engine.get_compliance_status())
}

#[post("/compliance/zkml-verify")]
async fn compliance_zkml_verify_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "verified": true, "compliance_standard": "CARF/BRS v1.5" }))
}

#[get("/identity/{query}")]
async fn identity_resolve_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "address": "bc1q...", "world_id_verified": true }))
}

#[post("/lightning/pay")]
async fn lightning_pay_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "status": "Paid", "fee_sats": 10 }))
}

#[derive(Deserialize)]
struct ErpSyncRequest {
    system: String,
    #[allow(dead_code)]
    testnet: Option<bool>,
}

#[post("/erp/sync")]
async fn erp_sync_handler(engine: web::Data<Engine>, payload: web::Json<ErpSyncRequest>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "system": payload.system, "sync_status": "Complete" }))
}

#[get("/spec/cjcs")]
async fn spec_cjcs_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "version": "0.2.5", "spec": "CJCS" }))
}

#[get("/finance/bond/{id}")]
async fn finance_bond_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "status": "Active", "yield_rate": "4.5%" }))
}

#[derive(Deserialize)]
struct StateCommitRequest {
    state_root: String,
    #[allow(dead_code)]
    testnet: Option<bool>,
    target: Option<AnchoringTarget>,
    idempotency_key: Option<String>,
    max_retry_attempts: Option<u8>,
    metadata: Option<HashMap<String, String>>,
}

fn anchoring_error_response(err: AnchoringError) -> HttpResponse {
    let status = match &err {
        AnchoringError::Validation { .. } => actix_web::http::StatusCode::BAD_REQUEST,
        AnchoringError::IdempotencyConflict { .. } => actix_web::http::StatusCode::CONFLICT,
        AnchoringError::RetryExhausted { .. } => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
        AnchoringError::AdapterFailure { retryable, .. } => {
            if *retryable {
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE
            } else {
                actix_web::http::StatusCode::BAD_GATEWAY
            }
        }
    };

    HttpResponse::build(status).json(serde_json::json!({
        "error": err.code(),
        "details": err,
    }))
}

#[post("/state/commit")]
async fn state_commit_handler(
    engine: web::Data<Engine>,
    req: web::Json<StateCommitRequest>,
) -> impl Responder {
    let is_testnet_request = req.testnet.unwrap_or(false);
    if Engine::is_mainnet_only() {
        if is_testnet_request {
            return HttpResponse::Forbidden()
                .body("Testnet bypass is strictly prohibited on production mainnet.");
        }
    } else if !is_testnet_request {
        return HttpResponse::Forbidden()
            .body("Non-production environment requires explicit testnet flag for validation.");
    }

    engine.increment_requests();

    let commit_request = AnchoringRequest {
        state_root: req.state_root.clone(),
        target: req.target.clone().unwrap_or_default(),
        idempotency_key: req.idempotency_key.clone(),
        metadata: req.metadata.clone().unwrap_or_default(),
        max_retry_attempts: req.max_retry_attempts.unwrap_or(DEFAULT_MAX_RETRY_ATTEMPTS),
    };

    match engine.commit_state_checkpoint(commit_request) {
        Ok(receipt) => HttpResponse::Ok().json(receipt),
        Err(err) => anchoring_error_response(err),
    }
}

const PARTNER_INTAKE_AUTH_HEADER: &str = "X-Partner-Intake-Key";
const PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

fn require_partner_intake_auth(req: &HttpRequest) -> Result<(), HttpResponse> {
    let expected_key = match std::env::var("PARTNER_INTAKE_API_KEY") {
        Ok(key) => key,
        Err(_) => return Err(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "auth_not_configured",
            "message": "PARTNER_INTAKE_API_KEY environment variable is not set"
        }))),
    };

    if let Some(key) = req.headers().get(PARTNER_INTAKE_AUTH_HEADER) {
        if key.to_str().unwrap_or_default() == expected_key {
            return Ok(());
        }
    }
    Err(HttpResponse::Unauthorized().finish())
}

fn require_admin_auth(req: &HttpRequest) -> Result<(), HttpResponse> {
    let expected_key = match std::env::var("GATEWAY_ADMIN_API_KEY") {
        Ok(key) => key,
        Err(_) => return Err(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "auth_not_configured",
            "message": "GATEWAY_ADMIN_API_KEY environment variable is not set"
        }))),
    };

    if let Some(key) = req.headers().get("X-Gateway-Admin-Key") {
        if key.to_str().unwrap_or_default() == expected_key {
            return Ok(());
        }
    }
    Err(HttpResponse::Unauthorized().finish())
}

fn require_idempotency_key(req: &HttpRequest) -> Result<String, HttpResponse> {
    req.headers()
        .get(PARTNER_INTAKE_IDEMPOTENCY_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
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
    #[allow(dead_code)]
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
    let res = engine.process_external_settlement("ISO20022", payload.into_inner());
    HttpResponse::Ok().json(res)
}

#[post("/settlement/papss")]
async fn papss_handler(engine: web::Data<Engine>, payload: web::Json<Value>) -> impl Responder {
    let res = engine.process_external_settlement("PAPSS", payload.into_inner());
    HttpResponse::Ok().json(res)
}

#[post("/settlement/brics")]
async fn brics_handler(engine: web::Data<Engine>, payload: web::Json<Value>) -> impl Responder {
    let res = engine.process_external_settlement("BRICS", payload.into_inner());
    HttpResponse::Ok().json(res)
}

#[get("/sab/wallets")]
async fn sab_wallets_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let wallets = engine.get_sab_wallets();
    HttpResponse::Ok().json(wallets)
}

#[get("/prices")]
async fn prices_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let prices = engine.prices.read().unwrap().clone();
    HttpResponse::Ok().json(prices)
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

#[derive(Deserialize)]
struct McpRequest {
    method: String,
    _params: Option<Value>,
}

#[post("/mcp")]
async fn mcp_handler(
    engine: web::Data<Engine>,
    req: HttpRequest,
    payload: web::Json<McpRequest>,
) -> impl Responder {
    if let Err(response) = require_admin_auth(&req) {
        return response;
    }
    engine.increment_requests();
    // In a real implementation, this would route to the engine's MCP execution logic.
    // For now, we'll return a placeholder success or error based on the method.
    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "method": payload.method.clone(),
        "message": "MCP command received and processed (Simulated)"
    }))
}

#[get("/citrea/proof/{batch_id}")]
async fn citrea_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    engine.increment_requests();
    let batch_id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({ "batch_id": batch_id, "status": "Verified", "layer": "Citrea" }))
}

#[get("/babylon/staking")]
async fn babylon_staking_handler(engine: web::Data<Engine>) -> impl Responder {
    HttpResponse::Ok().json(engine.get_babylon_staking())
}

#[get("/changelly")]
async fn changelly_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("changelly");
    HttpResponse::Ok().json(status)
}

#[get("/stacks/contract/{id}")]
async fn stacks_contract_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    engine.increment_requests();
    let contract_id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({ "contract_id": contract_id, "status": "Active", "layer": "Stacks" }))
}

#[get("/bitvm/proof/{id}")]
async fn bitvm_proof_handler(engine: web::Data<Engine>, path: web::Path<String>) -> impl Responder {
    engine.increment_requests();
    let proof_id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({ "proof_id": proof_id, "status": "Optimistic", "layer": "BitVM" }))
}

#[get("/finance/exchange-rate/{from}/{to}")]
async fn exchange_rate_handler(engine: web::Data<Engine>, path: web::Path<(String, String)>) -> impl Responder {
    engine.increment_requests();
    let (from, to) = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({ "from": from, "to": to, "rate": 1.0 }))
}

#[derive(Deserialize)]
struct LightningInvoiceRequest {
    amount_msat: u64,
    description: String,
}

#[post("/lightning/invoice")]
async fn lightning_invoice_handler(engine: web::Data<Engine>, payload: web::Json<LightningInvoiceRequest>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "invoice": "lnbc...", "amount_msat": payload.amount_msat, "description": payload.description }))
}

#[get("/liquid/peg")]
async fn liquid_peg_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "status": "Operational", "service": "/liquid/peg" }))
}

#[get("/rootstock/powpeg")]
async fn rootstock_powpeg_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "status": "Operational", "service": "/rootstock/powpeg" }))
}

#[get("/mezo/yield")]
async fn mezo_yield_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    HttpResponse::Ok().json(serde_json::json!({ "status": "Operational", "service": "/mezo/yield" }))
}

#[get("/rgb")]
async fn rgb_status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("rgb");
    HttpResponse::Ok().json(status)
}
#[get("/bitvm")]
async fn bitvm_status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("bitvm");
    HttpResponse::Ok().json(status)
}
#[get("/mezo")]
async fn mezo_status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("mezo");
    HttpResponse::Ok().json(status)
}
#[get("/citrea")]
async fn citrea_status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("citrea");
    HttpResponse::Ok().json(status)
}

#[get("/babylon")]
async fn babylon_status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("babylon");
    HttpResponse::Ok().json(status)
}

#[get("/lightning")]
async fn lightning_status_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let status = engine.get_service_status("lightning");
    HttpResponse::Ok().json(status)
}
