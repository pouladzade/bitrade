//use bitrade::{config::app_config::load_config};
use bitrade::grpc::server::start_server;

#[tokio::main]
async fn main() {

    start_server("[::]:50020".to_string()).await.unwrap();
}
