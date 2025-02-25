//use bitrade::{config::app_config::load_config};
use bitrade::grpc::server::start_server;
use tracing_subscriber::{self, FmtSubscriber};

#[tokio::main]
async fn main() {
    // let subscriber = FmtSubscriber::builder()
    //     .with_max_level(tracing::Level::DEBUG) // Enable debug logs
    //     .finish();

    // tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    // tracing::debug!("This debug message will be visible now!");
    // tracing_subscriber::fmt::init(); // Initialize the default subscriber

    // tracing::debug!("This is a debug message!");

    start_server("[::]:50020".to_string()).await.unwrap();
}
