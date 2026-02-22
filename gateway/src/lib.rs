pub mod api;
pub mod engine;

use actix_web::{App, HttpServer, web, middleware::Logger};
use crate::engine::Engine;
use std::net::TcpListener;

pub async fn start_gateway_server(port: u16) -> std::io::Result<()> {
    let engine = web::Data::new(Engine::new());

    // Start background monitoring
    Engine::start_monitoring(engine.clone()).await;

    let host = "0.0.0.0";
    log::info!("Starting Conxian Gateway Service on {}:{}", host, port);

    let listener = TcpListener::bind((host, port))?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(engine.clone())
            .configure(api::config)
    })
    .listen(listener)?
    .run()
    .await
}
