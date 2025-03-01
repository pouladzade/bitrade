use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
pub struct Config {
    #[structopt(long = "database-url", env = "DATABASE_URL")]
    pub db_url: String,
    #[structopt(long = "pool-size", env = "DATABASE_POOL_SIZE", default_value = "3")]
    pub pool_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_url: Default::default(),
            pool_size: 3,
        }
    }
}
