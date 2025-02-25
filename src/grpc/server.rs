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

    if let Err(e) = Server::builder()
        .add_service(SpotServiceServer::new(SpotServiceImpl {
            market_manager: Arc::new(RwLock::new(MarketManager::new())),
        }))
        .serve(adr)
        .await
    {
        error!("failed to read from socket; err = {:?}", e);
    }

    Ok(())
}
