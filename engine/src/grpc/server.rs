use database::establish_connection_pool;
use database::repository::Repository;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::app_config::get_database_url;
use crate::grpc::spot::spot_service_server::SpotServiceServer;
use crate::{grpc::service::SpotServiceImpl, wallet::wallet_service::WalletService};
use log::{error, info};
use tonic::transport::Server;

use crate::market::market_manager::MarketManager;

pub async fn start_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let adr = address.parse().unwrap();
    info!("Bitrade Server listening on {}", address);

    let database_url = get_database_url();
    let pool_size = 10;
    let pool = establish_connection_pool(database_url, pool_size);
    let repository = Repository::new(pool);

    if let Err(e) = Server::builder()
        .add_service(SpotServiceServer::new(SpotServiceImpl {
            market_manager: Arc::new(RwLock::new(MarketManager::new(Arc::new(
                repository.clone(),
            )))),
            wallet_service: Arc::new(WalletService::new(Arc::new(repository))),
        }))
        .serve(adr)
        .await
    {
        error!("Failed to start server: {:?}", e);
    }

    Ok(())
}
