use anyhow::Result;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://postgres:mysecretpassword@localhost/postgres".to_string(),
                pool_size: 10,
            },
            server: ServerConfig {
                host: "[::]".to_string(),
                port: 50020,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        }
    }
}

pub fn load_config() -> Result<AppConfig> {
    let config = Config::builder()
        // Start with default values
        .add_source(File::with_name("config/default").required(false))
        // Add local config file
        .add_source(File::with_name("config/local").required(false))
        // Add environment variables with prefix "BITRADE_"
        .add_source(Environment::with_prefix("BITRADE").separator("_"))
        .build()?;

    let app_config: AppConfig = config.try_deserialize()?;
    Ok(app_config)
}

pub fn get_database_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:mysecretpassword@localhost/postgres".to_string())
}

pub fn get_server_address() -> String {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "[::]".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "50020".to_string())
        .parse::<u16>()
        .unwrap_or(50020);
    format!("{}:{}", host, port)
}
