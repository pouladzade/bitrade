use spot_query::server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server("0.0.0.0:50051".to_string()).await
}
