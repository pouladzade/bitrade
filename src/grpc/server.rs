use database::persistence::ThreadSafePersistence;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::grpc::service::SpotServiceImpl;
use crate::grpc::spot::spot_service_server::SpotServiceServer;
use log::{error, info};
use tonic::transport::Server;

use crate::market::market_manager::MarketManager;

pub async fn start_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let adr = address.parse().unwrap();
    info!("P2P Server listening on {}", address);
    let database_url = "postgres://postgres:mysecretpassword@localhost/postgres";
    let pool_size = 10;
    if let Err(e) = Server::builder()
        .add_service(SpotServiceServer::new(SpotServiceImpl {
            market_manager: Arc::new(RwLock::new(MarketManager::new())),
            persist: ThreadSafePersistence::new(database_url.to_string(), pool_size),
        }))
        .serve(adr)
        .await
    {
        error!("failed to read from socket; err = {:?}", e);
    }

    Ok(())
}
