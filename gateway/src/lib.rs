pub mod api;
pub mod engine;

use actix_web::{App, HttpServer, web, middleware::Logger};
use crate::engine::Engine;
use std::net::TcpListener;

pub async fn start_gateway_server(port: u16) -> std::io::Result<()> {
    // Initialize logger if not already initialized
    // Note: In the unified binary, env_logger might be initialized by the main app,
    // so we should be careful. For now, we assume standard log macros are used.
    
    let engine = web::Data::new(Engine::new());
    let host = "0.0.0.0";

    log::info!("Starting Conxian Gateway Service on {}:{}", host, port);

    // We use a TcpListener to bind early and verify availability
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
