use std::sync::Arc;
use tokio::sync::RwLock;

use crate::grpc::spot::spot_service_server::SpotServiceServer;
use crate::{config::app_config::AppConfig, grpc::service::SpotServiceImpl};
use log::{error, info};
use tonic::transport::Server;

use crate::market::market::Market;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>>
where
{
    // defining address for our service

    let address = "[::]:50020".parse().unwrap();
    info!("P2P Server listening on {}", address);

    if let Err(e) = Server::builder()
        .add_service(SpotServiceServer::new(SpotServiceImpl {
            market: Arc::new(RwLock::new(Market::new(20))),
        }))
        .serve(address)
        .await
    {
        error!("failed to read from socket; err = {:?}", e);
    }

    Ok(())
}
