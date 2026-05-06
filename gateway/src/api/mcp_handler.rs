use crate::api::require_admin_auth;
use crate::engine::mcp::McpManager;
use crate::engine::Engine;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct McpRequest {
    pub method: String,
    pub params: Value,
}

#[post("/mcp")]
pub async fn mcp_handler(
    engine: web::Data<Engine>,
    http_req: HttpRequest,
    req: web::Json<McpRequest>,
) -> impl Responder {
    if let Err(response) = require_admin_auth(&http_req) {
        return response;
    }

    let mcp = McpManager::new(engine.into_inner());

    match req.method.as_str() {
        "tools/list" => {
            let tools = vec![
                mcp.get_telemetry_tool(),
                mcp.get_proof_tool(),
                mcp.get_yield_oracle_tool(),
                mcp.get_industrial_intents_tool(),
                mcp.get_draft_intent_tool(),
            ];
            HttpResponse::Ok().json(serde_json::json!({ "tools": tools }))
        }
        "tools/call" => {
            let tool_name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = req
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            let response = mcp.handle_call(tool_name, arguments).await;
            HttpResponse::Ok().json(response)
        }
        _ => HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Unsupported MCP method" })),
    }
}
