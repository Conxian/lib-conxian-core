with open('gateway/src/api/mod.rs', 'r') as f:
    content = f.read()

# Register the new service
content = content.replace(
    '.service(changelly_handler)',
    '.service(changelly_handler)\n            .service(changelly_rate_handler)'
)

# Update health handler
new_health_handler = """#[get("/health")]
async fn health_handler(engine: web::Data<Engine>) -> impl Responder {
    if engine.is_healthy() {
        HttpResponse::Ok().json(serde_json::json!({ "status": "healthy", "engine": "active" }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({ "status": "degraded", "engine": "stale" }))
    }
}"""

import re
content = re.sub(r'#\[get\("/health"\)\]\s*async\s*fn\s*health_handler\(.*?\)\s*->\s*impl\s*Responder\s*\{.*?\}', new_health_handler, content, flags=re.DOTALL)

# Add Changelly rate handler
new_handlers = """
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
"""

content += new_handlers

with open('gateway/src/api/mod.rs', 'w') as f:
    f.write(content)
