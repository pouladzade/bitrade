use std::env;

use config::{Config, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub grpc: GrpcConfig,
    pub database: DatabaseConfig,
    pub log: LogConfig,
    pub features: FeatureFlags,
    pub market: MarketConfig, // Add markets field
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct PrecisionConfig {
    pub amount: u32,
    pub price: u32,
    pub fee: u32,
}
#[derive(Debug, Deserialize)]
pub struct MinMarketConfig {
    pub amount: u32,
}
#[derive(Debug, Deserialize)]
pub struct MarketConfig {
    pub name: String,
    pub base: String,
    pub quote: String,
    pub precision: PrecisionConfig,
    pub min: MinMarketConfig,
}

#[derive(Debug, Deserialize)]
pub struct GrpcConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConnections {
    pub max: u32,
}
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub connections: DatabaseConnections,
}

#[derive(Debug, Deserialize)]
pub struct FeatureFlags {
    pub analytics: bool,
    pub cache: bool,
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    // Load .env file
    dotenv().ok();

    // Get the run mode (e.g., development, production)
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    // Build the configuration using the builder pattern
    let conf = Config::builder()
        // Load default configuration file
        .add_source(File::with_name("config/default").required(false))
        // Load environment-specific configuration file
        .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
        // Load environment variables with prefix "APP" and separator "_"
        .add_source(Environment::with_prefix("APP").separator("_"))
        .build()?;

    // Deserialize into AppConfig
    conf.try_deserialize()
}
