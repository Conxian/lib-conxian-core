pub mod api;
pub mod engine;

use crate::engine::Engine;
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::net::TcpListener;
use std::sync::Arc;

pub async fn start_gateway_server(port: u16) -> std::io::Result<()> {
    let engine = Arc::new(Engine::new());
    engine.initialize();

    let engine_data = web::Data::from(Arc::clone(&engine));

    // Start background monitoring
    Engine::start_monitoring(Arc::clone(&engine)).await;
    // Engine::poll_support(Arc::clone(Engine::poll_support(Arc::clone(&engine)).await;engine)).await;
    Engine::broadcast_intents(Arc::clone(&engine)).await;

    let host = "0.0.0.0";
    log::info!("Starting Conxian Gateway Service on {}:{}", host, port);

    let listener = TcpListener::bind((host, port))?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(engine_data.clone())
            .configure(api::config)
    })
    .listen(listener)?
    .run()
    .await
}
