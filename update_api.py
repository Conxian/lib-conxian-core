with open('gateway/src/api/mod.rs', 'r') as f:
    content = f.read()

content = content.replace(
    '.service(b2network_handler)',
    '.service(b2network_handler)\n            .service(citrea_handler)\n            .service(bitlayer_handler)\n            .service(prices_handler)'
)

new_handlers = """
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
"""

content += new_handlers

with open('gateway/src/api/mod.rs', 'w') as f:
    f.write(content)
