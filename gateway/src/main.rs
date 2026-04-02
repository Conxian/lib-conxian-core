use conxian_gateway::start_gateway_server;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger for standalone run
    env::set_var("RUST_LOG", "actix_web=info,conxian_gateway=debug");
    env_logger::init();

    let port: u16 = 8080;

    start_gateway_server(port).await
}
