use bitrade::{config::app_config::load_config, grpc::server::start_server};

#[tokio::main]
async fn main() {
    // let config = load_config().unwrap_or_else(|err| {
    //     eprintln!("Failed to load config: {}", err);
    //     std::process::exit(1);
    // });

    // println!("gRPC Server: {}:{}", config.grpc.host, config.grpc.port);
    // println!("Database URL: {:?}", config.database);
    // println!("Log Level: {}", config.log.level);
    // println!("Analytics Enabled: {:?}", config.features.analytics);
    // println!("Cache Enabled: {:?}", config.features.cache);

    // let market = &config.market;
    // println!("Market: {:?}", market);

    start_server("[::]:50020".to_string()).await.unwrap();
}
