use std::sync::Arc;
use tokio::sync::RwLock;

use crate::grpc::spot::spot_service_server::SpotServiceServer;
use crate::grpc::service::SpotServiceImpl;
use log::{error, info};
use tonic::transport::Server;

use crate::market::market::Market;

pub async fn start_server(address:String) -> Result<(), Box<dyn std::error::Error>>
{
    let adr = address.parse().unwrap();
    info!("P2P Server listening on {}", address);

    if let Err(e) = Server::builder()
        .add_service(SpotServiceServer::new(SpotServiceImpl {
            market: Arc::new(RwLock::new(Market::new(20))),
        }))
        .serve(adr)
        .await
    {
        error!("failed to read from socket; err = {:?}", e);
    }

    Ok(())
}
