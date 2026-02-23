with open('gateway/src/api/mod.rs', 'r') as f:
    content = f.read()

new_handler = """#[get("/compliance")]
async fn compliance_handler(engine: web::Data<Engine>) -> impl Responder {
    engine.increment_requests();
    let compliance = engine.get_compliance_status();
    HttpResponse::Ok().json(compliance)
}"""

import re
content = re.sub(r'#\[get\("/compliance"\)\]\s*async\s*fn\s*compliance_handler\(engine:\s*web::Data<Engine>\)\s*->\s*impl\s*Responder\s*\{.*?\}', new_handler, content, flags=re.DOTALL)

with open('gateway/src/api/mod.rs', 'w') as f:
    f.write(content)
