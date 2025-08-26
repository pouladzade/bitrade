use bitrade::{config::app_config::get_server_address, grpc::server::start_server};
use env_logger;
use log::{error, info};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    info!("Starting Bitrade Matching Engine...");

    let server_address = get_server_address();
    info!("Server will listen on: {}", server_address);

    match start_server(server_address).await {
        Ok(_) => info!("Server stopped gracefully"),
        Err(e) => error!("Server error: {}", e),
    }
}
