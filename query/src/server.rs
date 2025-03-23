use database::establish_connection_pool;
use database::repository::Repository;

use crate::service::SpotQueryServiceImp;
use crate::spot_query::spot_query_service_server::SpotQueryServiceServer;
use log::{error, info};
use tonic::transport::Server;

pub async fn start_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let adr = address.parse().unwrap();
    info!("P2P Server listening on {}", address);
    let database_url = "postgres://postgres:mysecretpassword@localhost/postgres";
    let pool_size = 10;
    let pool = establish_connection_pool(database_url.to_string(), pool_size);
    let repository = Repository::new(pool);
    if let Err(e) = Server::builder()
        .add_service(SpotQueryServiceServer::new(SpotQueryServiceImp::new(
            repository,
        )))
        .serve(adr)
        .await
    {
        error!("failed to read from socket; err = {:?}", e);
    }

    Ok(())
}
