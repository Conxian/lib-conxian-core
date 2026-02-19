mod api;
mod engine;

use actix_web::{App, HttpServer, web, middleware::Logger};
use crate::engine::Engine;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env::set_var("RUST_LOG", "actix_web=info,conxian_gateway=debug");
    env_logger::init();

    let engine = web::Data::new(Engine::new());

    let host = "0.0.0.0";
    let port = 8080;

    log::info!("Starting Conxian Gateway on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(engine.clone())
            .configure(api::config)
    })
    .bind((host, port))?
    .run()
    .await
}
